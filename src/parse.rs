use nom::error::{ErrorKind as NomErrorKind, ParseError as NomParseError};

pub type Input<'a> = &'a str;
pub type Result<'a, T> = nom::IResult<Input<'a>, T, Error<Input<'a>>>;

#[derive(Debug)]
pub struct Error<I> {
    pub errors: Vec<(I, NomErrorKind)>,
}

impl<I> NomParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: NomErrorKind) -> Self {
        let errors = vec![(input, kind)];
        Self { errors }
    }

    fn append(input: I, kind: NomErrorKind, mut other: Self) -> Self {
        other.errors.push((input, kind));
        other
    }
}
