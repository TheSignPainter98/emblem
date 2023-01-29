use crate::parser::{
    lexer::{LexicalError, Tok},
    point::Point,
};
use lalrpop_util::ParseError as LalrpopParseError;
use std::error;
use std::fmt::Display;
use std::io;
use std::ffi::OsString;

pub type LalrpopError<'i> = LalrpopParseError<Point<'i>, Tok<'i>, LexicalError<'i>>;

#[derive(Debug)]
pub enum Error<'i> {
    StringConversionError(OsStringConversionError),
    FilesystemError(io::Error),
    ParseError(LalrpopError<'i>),
}

impl error::Error for Error<'_> {}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StringConversionError(e) => e.fmt(f),
            Self::FilesystemError(e) => e.fmt(f),
            Self::ParseError(e) => e.fmt(f),
        }
    }
}

impl From<OsStringConversionError> for Box<Error<'_>> {
    fn from(err: OsStringConversionError) -> Self {
        Box::new(Error::StringConversionError(err))
    }
}

impl From<io::Error> for Box<Error<'_>> {
    fn from(err: io::Error) -> Self {
        Box::new(Error::FilesystemError(err))
    }
}

impl<'i> From<LalrpopError<'i>> for Box<Error<'i>> {
    fn from(err: LalrpopError<'i>) -> Self {
        Box::new(Error::ParseError(err))
    }
}

#[cfg(test)]
impl<'i> Error<'i> {
    pub fn string_conversion_error(&self) -> Option<&OsStringConversionError> {
        match self {
            Self::StringConversionError(e) => Some(e),
            _ => None,
        }
    }

    pub fn filesystem_error(&self) -> Option<&io::Error> {
        match self {
            Self::FilesystemError(e) => Some(e),
            _ => None,
        }
    }

    pub fn parse_error(&self) -> Option<&LalrpopError<'i>> {
        match self {
            Self::ParseError(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct OsStringConversionError {
    culprit: OsString,
}

impl OsStringConversionError {
    pub fn new(culprit: OsString) -> Self {
        Self { culprit }
    }
}

impl Display for OsStringConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not convert OS string: {:?}", self.culprit)
    }
}

impl error::Error for OsStringConversionError {}
