use indenter::Indented;
use nom::error::{ErrorKind as NomErrorKind, ParseError as NomParseError};
use std::fmt;
use std::fmt::Write;
use thiserror::Error;

pub type Input<'a> = &'a str;
pub type Result<'a, T> = nom::IResult<Input<'a>, T, Error>;

#[derive(Debug, Error)]
pub enum Kind {
    Nom,
    #[error(transparent)]
    ParseFloat {
        source: std::num::ParseFloatError,
    },
}

impl<'a> From<std::num::ParseFloatError> for Error {
    fn from(source: std::num::ParseFloatError) -> Self {
        let error = Kind::ParseFloat { source };
        Self {
            error: jane_eyre::ErrReport::from(error),
            nom_errors: vec![],
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Nom => write!(f, "unable to parse fields as an LC"),
            Kind::ParseFloat { source } => fmt::Display::fmt(&source, f),
        }
    }
}

#[derive(Error)]
pub struct Error
{
    pub error: jane_eyre::ErrReport,
    pub nom_errors: Vec<(String, NomErrorKind)>,
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.error, f)?;

        if !self.nom_errors.is_empty() {
            write!(f, "\nNom Context:")?;
        }

        for (ind, (i, k)) in self.nom_errors.iter().enumerate() {
            writeln!(f)?;
            write!(Indented::numbered(f, ind), "input={:?} NomKind={:?}", i, k)?;
        }

        Ok(())
    }
}

impl<I> NomParseError<I> for Error
where
    I: fmt::Display + fmt::Debug,
{
    fn from_error_kind(input: I, kind: NomErrorKind) -> Self {
        let nom_errors = vec![(input.to_string(), kind)];
        Self {
            error: jane_eyre::ErrReport::from(Kind::Nom),
            nom_errors,
        }
    }

    fn append(input: I, kind: NomErrorKind, mut other: Self) -> Self {
        other.nom_errors.push((input.to_string(), kind));
        other
    }
}
