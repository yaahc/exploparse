pub use eyre::*;

use indenter::Indented;
use nom::error::{VerboseErrorKind, ParseError as NomParseError};
use std::fmt;
use std::fmt::Write;
use std::any::{Any, TypeId};
use std::backtrace::Backtrace;
use std::backtrace::BacktraceStatus;
use tracing_error::{ExtractSpanTrace, SpanTrace, SpanTraceStatus};

pub struct ErrReport {
    inner: eyre::ErrReport<ExploContext>,
}

impl<E> From<E> for ErrReport
where
    eyre::ErrReport<ExploContext>: From<E>,
{
    fn from(error: E) -> Self {
        let inner = eyre::ErrReport::<ExploContext>::from(error);
        Self {
            inner
        }
    }
}

impl fmt::Debug for ErrReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

pub struct ExploContext {
    backtrace: Backtrace,
    span_trace: SpanTrace,
    nom_errors: Vec<(String, VerboseErrorKind)>,
}

impl EyreContext for ExploContext {
    fn default(_: &(dyn std::error::Error + 'static)) -> Self {
        Self {
            backtrace: Backtrace::capture(),
            span_trace: SpanTrace::capture(),
            nom_errors: Vec::new(),
        }
    }

    fn member_ref(&self, typeid: TypeId) -> Option<&dyn Any> {
        if typeid == TypeId::of::<Backtrace>() {
            Some(&self.backtrace)
        } else if typeid == TypeId::of::<SpanTrace>() {
            Some(&self.span_trace)
        } else {
            None
        }
    }

    fn display(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "{}", error)?;

        if f.alternate() {
            for cause in Chain::new(error).skip(1) {
                write!(f, ": {}", cause)?;
            }
        }

        Ok(())
    }

    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        if f.alternate() {
            return core::fmt::Debug::fmt(error, f);
        }

        let errors = Chain::new(error)
            .rev()
            .filter(|e| e.span_trace().is_none())
            .enumerate();

        for (n, error) in errors {
            writeln!(f)?;
            write!(Indented::numbered(f, n), "{}", error)?;
        }

        if !self.nom_errors.is_empty() {
            write!(f, "\n\nNom Context:")?;
        }

        for (ind, (i, k)) in self.nom_errors.iter().enumerate() {
            writeln!(f)?;
            write!(Indented::numbered(f, ind), "{:?} input={:?}", k, i)?;
        }

        let span_trace = &self.span_trace;

        match span_trace.status() {
            SpanTraceStatus::CAPTURED => write!(f, "\n\nSpan Trace:\n{}", span_trace)?,
            SpanTraceStatus::UNSUPPORTED => write!(f, "\n\nWarning: SpanTrace capture is Unsupported.\nEnsure that you've setup an error layer and the versions match")?,
            _ => (),
        }

        let backtrace = &self.backtrace;

        if let BacktraceStatus::Captured = backtrace.status() {
            write!(f, "\n\nStack Backtrace:\n{}", backtrace)?;
        }

        Ok(())
    }
}

impl<'a> NomParseError<&'a str> for ErrReport
{
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        let mut inner: eyre::ErrReport<ExploContext> = eyre::eyre!("unable to parse fields as an LC");
        inner.context_mut().nom_errors.push((input.to_string(), VerboseErrorKind::Nom(kind)));
        Self {
            inner
        }
    }

    fn from_char(input: &'a str, c: char) -> Self {
        let mut inner: eyre::ErrReport<ExploContext> = eyre::eyre!("Found unexpected character");
        inner.context_mut().nom_errors.push((input.to_string(), VerboseErrorKind::Char(c)));
        Self {
            inner
        }
    }

    fn append(input: &'a str, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.inner.context_mut().nom_errors.push((input.to_string(), VerboseErrorKind::Nom(kind)));
        other
    }

    fn add_context(input: &'a str, ctx: &'static str, mut other: Self) -> Self {
        other.inner.context_mut().nom_errors.push((input.to_string(), VerboseErrorKind::Context(ctx)));
        other
    }
}
