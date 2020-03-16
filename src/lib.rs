#![feature(error_iter)]

use nom::bytes::complete::{is_a, take, take_while, take_while_m_n};
use nom::character::complete::anychar;
use nom::combinator::{map, map_res, opt, verify};
use nom::error::context;
use tracing::{span, Level, instrument};

mod parse;

#[derive(Debug, PartialEq)]
pub struct Genre<'s>(&'s str);

#[derive(Debug, PartialEq)]
pub struct Second(f64);

#[derive(Debug, PartialEq)]
pub struct Third<'s> {
    has_dot: bool,
    body: &'s str,
}

#[derive(Debug, PartialEq)]
pub struct Year {
    year: u16,
    suffix: Option<char>,
}

#[derive(Debug, PartialEq)]
pub struct Note<'s>(&'s str);

#[derive(Debug, PartialEq)]
pub struct LC<'s> {
    pub genre: Genre<'s>,
    pub second: Second,
    pub third: Third<'s>,
    pub fourth: Option<Third<'s>>,
    pub year: Option<Year>,
    pub note: Option<Note<'s>>, // Note bits at the end
}

use std::fmt;
impl<'s> fmt::Display for LC<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.genre.0)?;
        write!(f, " {}", self.second.0)?;

        write!(f, " ")?;
        write!(f, "{}", self.third)?;

        if let Some(ref fourth) = self.fourth {
            write!(f, " ")?;
            write!(f, "{}", fourth)?;
        }

        if let Some(Year {
            ref year,
            ref suffix,
        }) = self.year
        {
            write!(f, " ")?;
            write!(f, "{}", year)?;

            if let Some(ref suffix) = suffix {
                write!(f, "{}", suffix)?;
            }
        }

        if let Some(ref note) = self.note {
            write!(f, " ")?;
            write!(f, "{}", note.0)?;
        }

        Ok(())
    }
}

impl<'s> fmt::Display for Third<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has_dot {
            write!(f, ".")?;
        }
        write!(f, "{}", self.body)?;

        Ok(())
    }
}

impl<'a> Genre<'a> {
    #[instrument]
    fn parse_genre(i: &'a str) -> parse::Result<'a, Self> {
        context(
            "Genre",
            map(take_while_m_n(1, 2, nom::AsChar::is_alpha), Genre),
        )(i)
    }
}

impl Second {
    #[instrument]
    fn parse_second(i: parse::Input) -> parse::Result<Self> {
        let mut seen_dot = false;
        let mut prev = None;
        let mut end = None;
        for (ind, c) in i.chars().enumerate() {
            match c {
                '.' => {
                    if seen_dot {
                        end = Some(ind);
                        break;
                    } else {
                        seen_dot = true;
                    }
                }
                'A'..='Z' => {
                    if prev == Some('.') {
                        end = Some(ind - 1);
                        break;
                    } else {
                        end = Some(ind);
                        break;
                    }
                }
                _ => (),
            }
            prev = Some(c);
        }

        let end = end.unwrap_or_else(|| i.len());

        let after = &i[end..];
        let second = &i[..end];
        let input = second
            .replace(" ", "");

        let span = span!(Level::ERROR, "Second::parse_float", ?input);
        let _guard = span.enter();

        let second = input.parse()
            .map_err(parse::Error::from)
            .map_err(nom::Err::Error)?;

        Ok((after, Second(second)))
    }
}

impl<'s> Third<'s> {
    #[instrument]
    fn parse_third(i: parse::Input<'s>) -> parse::Result<'s, Self> {
        let (i, _) = opt(is_a(" "))(i)?;
        let (i, has_dot) = map(opt(nom::character::complete::char('.')), |dot| {
            dot.is_some()
        })(i)?;
        let (i, _) = opt(is_a(" "))(i)?;
        let (i, body) = context(
            "Third",
            verify(take_while(|c: char| c.is_alphanumeric()), |s: &str| {
                s.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false)
            }),
        )(i)?;

        Ok((i, Third { has_dot, body }))
    }
}

impl Year {
    #[instrument]
    fn parse_year(i: parse::Input) -> parse::Result<Self> {
        let (i, _) = opt(is_a(" "))(i)?;
        let (i, year) = map_res(take(4usize), str::parse)(i)?;
        let (i, suffix) = opt(verify(anychar, |c| c.is_alphabetic()))(i)?;

        Ok((i, Year { year, suffix }))
    }
}

impl<'s> Note<'s> {
    // Implement note pieces here. Read whole string at the end and hold data
    #[instrument]
    fn parse_note(i: parse::Input<'s>) -> parse::Result<'s, Self> {
        let i = i.trim();
        if i.is_empty() {
            Err(nom::Err::Error(parse::Error {
                error: jane_eyre::ErrReport::from(parse::Kind::Nom),
                nom_errors: vec![(i.to_string(), nom::error::ErrorKind::Eof)],
            }))
        } else {
            let note = Note(i);
            Ok((i, note))
        }
    }
}

impl<'a> LC<'a> {
    pub fn maybe_parse(i: &'a str) -> Result<Option<LC<'a>>, parse::Error> {
        if i.is_empty() {
            Ok(None)
        } else {
            // Shows fixed LC otherwise
            let (_, lc) = LC::parse_lc(i)
                .map_err(|e| match e {
                    nom::Err::Error(e) | nom::Err::Failure(e) => e,
                    _ => unreachable!(),
                })?;
            Ok(Some(lc))
        }
    }

    #[instrument]
    pub fn parse_lc(i: parse::Input<'a>) -> parse::Result<'a, Self> {
        let (i, genre) = Genre::parse_genre(i)?;
        let (i, second) = Second::parse_second(i)?;
        let (i, third) = Third::parse_third(i)?;
        let (i, fourth) = opt(Third::parse_third)(i)?;
        let (i, year) = opt(Year::parse_year)(i)?;
        let (i, note) = opt(Note::parse_note)(i)?;

        Ok((
            i,
            Self {
                genre,
                second,
                third,
                fourth,
                year,
                note,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full() {
        let lc = "TD 224 .C3 C3723 2009";
        let expected = LC {
            genre: Genre("TD"),
            second: Second(224.0),
            third: Third {
                has_dot: true,
                body: "C3",
            },
            fourth: Some(Third {
                has_dot: false,
                body: "C3723",
            }),
            year: Some(Year {
                year: 2009,
                suffix: None,
            }),
            note: None,
        };
        let (_, lc) = LC::parse_lc(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn test_first() {
        let lc = "GB 658 .C43 2005";
        let expected = LC {
            genre: Genre("GB"),
            second: Second(658.0),
            third: Third {
                has_dot: true,
                body: "C43",
            },
            fourth: None,
            year: Some(Year {
                year: 2005,
                suffix: None,
            }),
            note: None,
        };
        let (_, lc) = LC::parse_lc(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn test_suffix() {
        let lc = "GC 21.5 .S56 1988b";
        let expected = LC {
            genre: Genre("GC"),
            second: Second(21.5),
            third: Third {
                has_dot: true,
                body: "S56",
            },
            fourth: None,
            year: Some(Year {
                year: 1988,
                suffix: Some('b'),
            }),
            note: None,
        };
        let (_, lc) = LC::parse_lc(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn test_optspace() {
        let lc = "TD224.C3 C3723 2004";
        let expected = LC {
            genre: Genre("TD"),
            second: Second(224.0),
            third: Third {
                has_dot: true,
                body: "C3",
            },
            fourth: Some(Third {
                has_dot: false,
                body: "C3723",
            }),
            year: Some(Year {
                year: 2004,
                suffix: None,
            }),
            note: None,
        };

        let (_, lc) = LC::parse_lc(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn extra_space() {
        let lc = "QC 920 .Z38 2009 ";
        let expected = LC {
            genre: Genre("QC"),
            second: Second(920.0),
            third: Third {
                has_dot: true,
                body: "Z38",
            },
            fourth: None,
            year: Some(Year {
                year: 2009,
                suffix: None,
            }),
            note: None,
        };

        let (_, lc) = LC::parse_lc(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    // Row { lc: "QC 183 .G675" }
    #[test]
    fn missing_year() {
        let lc = "QC 183 .G675";
        let expected = LC {
            genre: Genre("QC"),
            second: Second(183.0),
            third: Third {
                has_dot: true,
                body: "G675",
            },
            fourth: None,
            year: None,
            note: None,
        };

        let (_, lc) = LC::parse_lc(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    // Row { lc: "HD 1695 .K55 .V5 2010" }
    #[test]
    fn double_dot() {
        let lc = "HD 1695 .K55 .V5 2010";
        let expected = LC {
            genre: Genre("HD"),
            second: Second(1695.0),
            third: Third {
                has_dot: true,
                body: "K55",
            },
            fourth: Some(Third {
                has_dot: true,
                body: "V5",
            }),
            year: Some(Year {
                year: 2010,
                suffix: None,
            }),
            note: None,
        };

        let (_, lc) = LC::parse_lc(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    // Row { lc: "HD 1695 .55 .K55 .V5 2010" }
    #[test]
    fn space_in_float() {
        let lc = "HD 1695 .55 .K55 .V5 2010";
        let expected = LC {
            genre: Genre("HD"),
            second: Second(1695.55),
            third: Third {
                has_dot: true,
                body: "K55",
            },
            fourth: Some(Third {
                has_dot: true,
                body: "V5",
            }),
            year: Some(Year {
                year: 2010,
                suffix: None,
            }),
            note: None,
        };

        let (_, lc) = LC::parse_lc(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn offset_dots() {
        let lc = "HD 1695 .55. K55. V5 2010";
        let expected = LC {
            genre: Genre("HD"),
            second: Second(1695.55),
            third: Third {
                has_dot: true,
                body: "K55",
            },
            fourth: Some(Third {
                has_dot: true,
                body: "V5",
            }),
            year: Some(Year {
                year: 2010,
                suffix: None,
            }),
            note: None,
        };

        let (_, lc) = LC::parse_lc(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn round_trip() {
        let lc = "HD 1695 .55. K55. V5 2010";
        let expected = "HD 1695.55 .K55 .V5 2010";
        let lc = LC::maybe_parse(lc).unwrap().unwrap().to_string();
        assert_eq!(expected, lc);
    }

    #[test]
    fn round_trip_no_float() {
        let lc = "HD 1695 .K55 .V5 2010";
        let expected = "HD 1695 .K55 .V5 2010";
        let lc = LC::maybe_parse(lc).unwrap().unwrap().to_string();
        assert_eq!(expected, lc);
    }

    // Row { lc: "TD 225 .S25 H26x 2002" }
    #[test]
    fn trailing_char() {
        let lc = "TD 225 .S25 H26x 2002";
        dbg!(lc);
        let expected = LC {
            genre: Genre("TD"),
            second: Second(225.0),
            third: Third {
                has_dot: true,
                body: "S25",
            },
            fourth: Some(Third {
                has_dot: false,
                body: "H26x",
            }),
            year: Some(Year {
                year: 2002,
                suffix: None,
            }),
            note: None,
        };

        let (_, lc) = LC::parse_lc(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    // Row { lc: "G 4364 .R6 .S6C3 2006" }
    #[test]
    fn midstring_char() {
        let lc = "G 4364 .R6 .S6C3 2006";
        dbg!(lc);
        let expected = LC {
            genre: Genre("G"),
            second: Second(4364.0),
            third: Third {
                has_dot: true,
                body: "R6",
            },
            fourth: Some(Third {
                has_dot: true,
                body: "S6C3",
            }),
            year: Some(Year {
                year: 2006,
                suffix: None,
            }),
            note: None,
        };

        let (_, lc) = LC::parse_lc(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn with_note() {
        let lc = "LB 1028 .L43 2000 vol. tg ";
        let expected_round_trip = "LB 1028 .L43 2000 vol. tg";
        dbg!(lc);
        let expected = LC {
            genre: Genre("LB"),
            second: Second(1028.0),
            third: Third {
                has_dot: true,
                body: "L43",
            },
            fourth: None,
            year: Some(Year {
                year: 2000,
                suffix: None,
            }),
            note: Some(Note("vol. tg")),
        };

        let lc = LC::maybe_parse(lc).unwrap().unwrap();
        assert_eq!(&expected, dbg!(&lc));
        let round_trip = lc.to_string();
        assert_eq!(expected_round_trip, round_trip);
    }

    #[test]
    fn mid_dot() {
        let lc_string = "QB 46 .L744 v.82 2000";
        dbg!(lc_string);
        let expected = LC {
            genre: Genre("QB"),
            second: Second(46.0),
            third: Third {
                has_dot: true,
                body: "L744",
            },
            fourth: Some(Third {
                has_dot: true,
                body: "v82",
            }),
            year: Some(Year {
                year: 2000,
                suffix: None,
            }),
            note: None,
        };

        let lc = LC::maybe_parse(lc_string).unwrap().unwrap();
        assert_eq!(&expected, dbg!(&lc));
        let round_trip = lc.to_string();
        assert_eq!(lc_string, round_trip);
    }

    // Test case no longer necessary
    // #[test]
    // //Row "Circ. desk"
    // fn only_text() {
    //     let lc = "Circ. desk";
    //     dbg!(lc);
    //     let expected = LC {
    //         genre: Genre(None),
    //         second: Second(None),
    //         third: Third(None),
    //         fourth: None,
    //         year: None,
    //         note: "Circ. desk",
    //     };

    //     let (_, lc) = LC::parse_lc(lc).unwrap();
    //     assert_eq!(expected,dbg!(lc));
    // }
}
