use nom::bytes::complete::{is_a, take, take_while, take_while_m_n};
use nom::character::complete::anychar;
use nom::combinator::{map, map_res, opt, verify};
use nom::error::context;

mod parse;

#[derive(Debug, PartialEq)]
pub struct Genre<'s>(&'s str);

#[derive(Debug, PartialEq)]
pub struct Second<'s>(&'s str);

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
    genre: Genre<'s>,
    second: Second<'s>,
    third: Third<'s>,
    fourth: Option<Third<'s>>,
    year: Option<Year>,
    note: Option<Note<'s>>, // Note bits at the end
}

impl<'a> Genre<'a> {
    fn parse(i: &'a str) -> parse::Result<'a, Self> {
        context(
            "Genre",
            map(take_while_m_n(1, 2, nom::AsChar::is_alpha), Genre),
        )(i)
    }
}

impl<'s> Second<'s> {
    fn parse(i: parse::Input<'s>) -> parse::Result<'s, Self> {
        let mut seen_dot = false;
        let mut prev = None;
        let mut end = None;
        for (ind, c) in i.chars().enumerate() {
            match c {
                '.' => if seen_dot {
                    end = Some(ind);
                    break;
                } else {
                    seen_dot = true;
                },
                'A'..='Z' => if prev == Some('.') {
                    end = Some(ind - 1);
                    break;
                } else {
                    end = Some(ind);
                    break;
                }
                _ => (),
            }
            prev = Some(c);
        }

        let end = end.ok_or_else(|| nom::Err::Error(parse::Error { errors: vec![(i, nom::error::ErrorKind::Eof)], }))?;

        let after = &i[end..];
        let second = &i[..end];
        let second = second.trim();

        Ok((after, Second(second)))
    }
}

impl<'s> Third<'s> {
    fn parse(i: parse::Input<'s>) -> parse::Result<'s, Self> {
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
    fn parse(i: parse::Input) -> parse::Result<Self> {
        let (i, _) = opt(is_a(" "))(i)?;
        let (i, year) = map_res(take(4usize), str::parse)(i)?;
        let (i, suffix) = opt(verify(anychar, |c| c.is_alphabetic()))(i)?;

        Ok((i, Year { year, suffix }))
    }
}

impl<'s> Note<'s> {
    // Implement note pieces here. Read whole string at the end and hold data
    fn last_but_not_least(i: parse::Input) -> parse::Result<Self> {
        if i.is_empty() {
                Err(nom::Err::Error(parse::Error { errors: vec![(i, nom::error::ErrorKind::Eof)], }))
        } else {
            let (_, note) = Note::last_but_not_least(i)?;
            Ok((i, note))
        }
    }    
}

impl<'a> LC<'a> {
    pub fn maybe_parse(i: &'a str) -> Result<Option<LC<'a>>, nom::Err<parse::Error<&'a str>>> {
        if i.is_empty() {
            Ok(None)
        } else {
            let (_, lc) = LC::parse(i)?;
            Ok(Some(lc))
        }
    }

    pub fn parse(i: parse::Input<'a>) -> parse::Result<'a, Self> {
        let (i, genre) = Genre::parse(i)?;
        let (i, second) = Second::parse(i)?;
        let (i, third) = Third::parse(i)?;
        let (i, fourth) = opt(Third::parse)(i)?;
        let (extra, year) = opt(Year::parse)(i)?;
        let (extra, note) = opt(Note::last_but_not_least)(i)?;
        if !extra.trim().is_empty() {
            Err(nom::Err::Failure(parse::Error {
                errors: vec![(extra, nom::error::ErrorKind::NonEmpty)],
            }))
        } else {
            Ok((
                extra,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full() {
        let lc = "TD 224 .C3 C3723 2009";
        let expected = LC {
            genre: Genre("TD"),
            second: Second("224"),
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
        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn test_first() {
        let lc = "GB 658 .C43 2005";
        let expected = LC {
            genre: Genre("GB"),
            second: Second("658"),
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
        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn test_suffix() {
        let lc = "GC 21.5 .S56 1988b";
        let expected = LC {
            genre: Genre("GC"),
            second: Second("21.5"),
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
        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn test_optspace() {
        let lc = "TD224.C3 C3723 2004";
        let expected = LC {
            genre: Genre("TD"),
            second: Second("224"),
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

        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn extra_space() {
        let lc = "QC 920 .Z38 2009 ";
        let expected = LC {
            genre: Genre("QC"),
            second: Second("920"),
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

        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    // Row { lc: "QC 183 .G675" }
    #[test]
    fn missing_year() {
        let lc = "QC 183 .G675";
        let expected = LC {
            genre: Genre("QC"),
            second: Second("183"),
            third: Third {
                has_dot: true,
                body: "G675",
            },
            fourth: None,
            year: None,
            note: None,
        };

        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    // Row { lc: "HD 1695 .K55 .V5 2010" }
    #[test]
    fn double_dot() {
        let lc = "HD 1695 .K55 .V5 2010";
        let expected = LC {
            genre: Genre("HD"),
            second: Second("1695"),
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

        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    // Row { lc: "HD 1695 .55 .K55 .V5 2010" }
    #[test]
    fn space_in_float() {
        let lc = "HD 1695 .55 .K55 .V5 2010";
        let expected = LC {
            genre: Genre("HD"),
            second: Second("1695 .55"),
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

        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    #[test]
    fn offset_dots() {
        let lc = "HD 1695 .55. K55. V5 2010";
        let expected = LC {
            genre: Genre("HD"),
            second: Second("1695 .55"),
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

        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    // Row { lc: "TD 225 .S25 H26x 2002" }
    #[test]
    fn trailing_char() {
        let lc = "TD 225 .S25 H26x 2002";
        dbg!(lc);
        let expected = LC {
            genre: Genre("TD"),
            second: Second("225"),
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

        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }

    // Row { lc: "G 4364 .R6 .S6C3 2006" }
    #[test]
    fn midstring_char() {
        let lc = "G 4364 .R6 .S6C3 2006";
        dbg!(lc);
        let expected = LC {
            genre: Genre("G"),
            second: Second("4364"),
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

        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }
}
