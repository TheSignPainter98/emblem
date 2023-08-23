use crate::{
    log::{
        messages::{UnexpectedEOF, UnexpectedToken},
        Log, Message,
    },
    parser::{
        self,
        lexer::{LexicalError, Tok},
        Location, Point,
    },
};
use lalrpop_util::ParseError as LalrpopParseError;
use std::error;
use std::ffi::OsString;
use std::fmt::Display;
use std::io;

pub type LalrpopError = LalrpopParseError<Point, Tok, Box<LexicalError>>;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Error {
    StringConversion(StringConversionError),
    Filesystem(io::Error),
    Parse(LalrpopError),
}

impl Message for parser::Error {
    fn log(self) -> Log {
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

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StringConversion(e) => e.fmt(f),
            Self::Filesystem(e) => e.fmt(f),
            Self::Parse(e) => e.fmt(f),
        }
    }
}

impl From<StringConversionError> for Box<Error> {
    fn from(err: StringConversionError) -> Self {
        Box::new(Error::StringConversion(err))
    }
}

impl From<io::Error> for Box<Error> {
    fn from(err: io::Error) -> Self {
        Box::new(Error::Filesystem(err))
    }
}

impl From<LalrpopError> for Box<Error> {
    fn from(err: LalrpopError) -> Self {
        Box::new(Error::Parse(err))
    }
}

#[cfg(test)]
impl Error {
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

    pub fn parse_error(&self) -> Option<&LalrpopError> {
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
