use crate::{
    log::{Message, messages::{UnexpectedEOF, UnexpectedToken}, Log},
    parser::{
        self,
        lexer::{LexicalError, Tok},
        Point,
        Location,
    },
};
use lalrpop_util::ParseError as LalrpopParseError;
use std::error;
use std::ffi::OsString;
use std::fmt::Display;
use std::io;

pub type LalrpopError<'i> = LalrpopParseError<Point<'i>, Tok<'i>, Box<LexicalError<'i>>>;

#[derive(Debug)]
pub enum Error<'i> {
    StringConversion(StringConversionError),
    Filesystem(io::Error),
    Parse(LalrpopError<'i>),
}

impl<'i> Message<'i> for parser::Error<'i> {
    fn log(self) -> Log<'i> {
        match self {
            parser::Error::StringConversion(e) => Log::error(e.to_string()),
            parser::Error::Filesystem(e) => Log::error(e.to_string()),
            parser::Error::Parse(e) => match e {
                LalrpopError::InvalidToken { location } => {
                    panic!("internal error: invalid token at {}", location)
                }
                LalrpopError::UnrecognizedEOF { location, expected } => {
                    UnexpectedEOF::new(location, expected).log()
                }
                LalrpopError::UnrecognizedToken {
                    token: (l, t, r),
                    expected,
                } => UnexpectedToken::new(Location::new(&l, &r), t, expected).log(),
                LalrpopError::ExtraToken { token: (l, t, r) } => panic!(
                    "internal error: extra token {} at {}",
                    t,
                    Location::new(&l, &r)
                ),
                LalrpopError::User { error } => error.log(),
            },
        }
    }
}

impl error::Error for Error<'_> {}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StringConversion(e) => e.fmt(f),
            Self::Filesystem(e) => e.fmt(f),
            Self::Parse(e) => e.fmt(f),
        }
    }
}

impl From<StringConversionError> for Box<Error<'_>> {
    fn from(err: StringConversionError) -> Self {
        Box::new(Error::StringConversion(err))
    }
}

impl From<io::Error> for Box<Error<'_>> {
    fn from(err: io::Error) -> Self {
        Box::new(Error::Filesystem(err))
    }
}

impl<'i> From<LalrpopError<'i>> for Box<Error<'i>> {
    fn from(err: LalrpopError<'i>) -> Self {
        Box::new(Error::Parse(err))
    }
}

#[cfg(test)]
impl<'i> Error<'i> {
    pub fn string_conversion_error(&self) -> Option<&StringConversionError> {
        match self {
            Self::StringConversion(e) => Some(e),
            _ => None,
        }
    }

    pub fn filesystem_error(&self) -> Option<&io::Error> {
        match self {
            Self::Filesystem(e) => Some(e),
            _ => None,
        }
    }

    pub fn parse_error(&self) -> Option<&LalrpopError<'i>> {
        match self {
            Self::Parse(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct StringConversionError {
    culprit: OsString,
}

impl StringConversionError {
    pub fn new(culprit: OsString) -> Self {
        Self { culprit }
    }
}

impl Display for StringConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not convert OS string: {:?}", self.culprit)
    }
}

impl error::Error for StringConversionError {}
