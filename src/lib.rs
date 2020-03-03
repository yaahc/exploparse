use nom::bytes::complete::{is_a, take, take_till, take_while, take_while_m_n};
use nom::character::complete::anychar;
use nom::combinator::{map, opt, verify};
use nom::error::context;

mod parse;

#[derive(Debug, PartialEq)]
pub struct Genre<'s>(&'s str);

#[derive(Debug, PartialEq)]
pub struct Second<'s>(&'s str);

#[derive(Debug, PartialEq)]
pub struct Third<'s>(&'s str);

#[derive(Debug, PartialEq)]
pub struct Fourth<'s>(&'s str);

#[derive(Debug, PartialEq)]
pub struct Year {
    year: u16,
    suffix: Option<char>,
}

#[derive(Debug, PartialEq)]
pub struct LC<'s> {
    genre: Genre<'s>,
    second: Second<'s>,
    third: Option<Third<'s>>,
    fourth: Option<Fourth<'s>>,
    year: Option<Year>,
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
        let until_char = take_till(|c: char| c.is_alphabetic() || c.is_whitespace());
        let until_dot = take_till(|c| c == '.');
        let (i, _) = opt(is_a(" "))(i)?;

        let (i_char, s) = until_char(i)?;
        if s.ends_with('.') {
            map(until_dot, Second)(i)
        } else {
            Ok((i_char, Second(s)))
        }
    }
}

impl<'a> Third<'a> {
    fn parse(i: parse::Input<'a>) -> parse::Result<'a, Self> {
        let (i, _) = opt(is_a(" "))(i)?;
        context(
            "Third",
            map(
                verify(take_while(|c| c != ' '), |s: &str| {
                    s.chars().next().map(|c| c == '.').unwrap_or(false)
                }),
                Third,
            ),
        )(i)
    }
}

impl<'a> Fourth<'a> {
    fn parse(i: parse::Input<'a>) -> parse::Result<'a, Self> {
        let (i, _) = is_a(" ")(i)?;
        context(
            "Fourth",
            map(
                verify(take_while(|c| c != ' '), |s: &str| {
                    s.chars().next().map(|c| !c.is_digit(10)).unwrap_or(false)
                }),
                Fourth,
            ),
        )(i)
    }
}

impl Year {
    fn parse(i: parse::Input) -> parse::Result<Self> {
        let (i, _) = is_a(" ")(i)?;
        let (i, year) = take(4usize)(i)?;
        let year = year.parse().unwrap();
        let (i, suffix) = opt(verify(anychar, |c| c.is_alphabetic()))(i)?;

        Ok((i, Year { year, suffix }))
    }
}

impl<'a> LC<'a> {
    pub fn maybe_parse(i: &'a str) -> Result<Option<LC<'a>>, nom::Err<parse::Error<&'a str>>> {
        if i.is_empty() {
            Ok(None)
        } else {
            LC::parse(i).map(|(_, lc)| Some(lc))
        }
    }

    pub fn parse(i: parse::Input<'a>) -> parse::Result<'a, Self> {
        let (i, genre) = Genre::parse(i)?;
        let (i, second) = Second::parse(i)?;
        let (i, third) = opt(Third::parse)(i)?;
        let (i, fourth) = opt(Fourth::parse)(i)?;
        let (i, year) = opt(Year::parse)(i)?;
        Ok((
            i,
            Self {
                genre,
                second,
                third,
                fourth,
                year,
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
            second: Second("224"),
            third: Some(Third(".C3")),
            fourth: Some(Fourth("C3723")),
            year: Some(Year {
                year: 2009,
                suffix: None,
            }),
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
            third: Some(Third(".C43")),
            fourth: None,
            year: Some(Year {
                year: 2005,
                suffix: None,
            }),
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
            third: Some(Third(".S56")),
            fourth: None,
            year: Some(Year {
                year: 1988,
                suffix: Some('b'),
            }),
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
            third: Some(Third(".C3")),
            fourth: Some(Fourth("C3723")),
            year: Some(Year {
                year: 2004,
                suffix: None,
            }),
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
            third: Some(Third(".Z38")),
            fourth: None,
            year: Some(Year {
                year: 2009,
                suffix: None,
            }),
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
            third: Some(Third(".G675")),
            fourth: None,
            year: None,
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
            third: Some(Third(".K55")),
            fourth: Some(Fourth(".V5")),
            year: Some(Year {
                year: 2010,
                suffix: None,
            }),
        };

        let (_, lc) = LC::parse(lc).unwrap();
        assert_eq!(expected, dbg!(lc));
    }
}
