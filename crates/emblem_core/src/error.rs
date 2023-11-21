use std::{borrow::Cow, error::Error as StdError, ffi::OsString, fmt::Display, io};

use crate::{log::LogId, parser::error::ParseError, FileName, Log};

#[derive(Debug)]
pub struct Error(Box<ErrorImpl>);

impl Error {
    fn new(error: ErrorImpl) -> Self {
        Self(Box::new(error))
    }

    pub fn context(self, context: impl Into<Cow<'static, str>>) -> Self {
        let context = context.into();
        Self::new(ErrorImpl::WithContext {
            context,
            cause: self,
        })
    }

    pub fn no_such_error_code(id: LogId) -> Self {
        Self::new(ErrorImpl::NoSuchErrorCode(id))
    }

    pub fn parse(file_name: FileName, cause: ParseError) -> Self {
        Self::new(ErrorImpl::ParseError { file_name, cause })
    }

    pub fn string_conversion(culprit: OsString) -> Self {
        Self::new(ErrorImpl::StringConversion { culprit })
    }

    pub fn too_many_errors(tot_errors: i32) -> Self {
        Self::new(ErrorImpl::TooManyErrors { tot_errors })
    }

    pub fn uncallable_listener(type_name: &'static str) -> Self {
        Self::new(ErrorImpl::UncallableListener { type_name })
    }
}

impl<T: Into<ErrorImpl>> From<T> for Error {
    fn from(cause: T) -> Self {
        Self::new(cause.into())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &*self.0 {
            ErrorImpl::WithContext { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, thiserror::Error)]
enum ErrorImpl {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("lua error: {0}")]
    Lua(#[from] mlua::Error),

    #[error("no such error code: {0}")]
    NoSuchErrorCode(LogId),

    #[error("cannot parse '{file_name}': {cause}")]
    ParseError {
        file_name: FileName,
        cause: ParseError,
    },

    #[error("cannot convert string to utf8: {}", culprit.to_string_lossy())]
    StringConversion { culprit: OsString },

    #[error("run aborted after {tot_errors}")]
    TooManyErrors { tot_errors: i32 },

    #[error("{type_name} is not callable")]
    UncallableListener { type_name: &'static str },

    #[error("{context}: {cause}")]
    WithContext {
        context: Cow<'static, str>,
        cause: Error,
    },
}

impl From<Error> for Log {
    fn from(_error: Error) -> Self {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use crate::{parser::Point, Context};

    use super::*;

    #[test]
    fn no_such_error_code() {
        assert_eq!(
            Error::no_such_error_code(LogId::Defined("E9999".into())).to_string(),
            "no such error code: E9999"
        );
    }

    #[test]
    fn parse() {
        let ctx = Context::test_new();
        let file_name = ctx.alloc_file_name("file-name-here");
        let file_content = ctx.alloc_file_content("hjkfdl fhdjk fdsaljkh");
        let point = Point::at_start_of(file_name.clone(), file_content);
        let err = Error::parse(file_name, ParseError::InvalidToken { location: point });
        assert_eq!(
            err.to_string(),
            "cannot parse 'file-name-here': Invalid token at 1:1"
        )
    }

    #[test]
    fn string_conversion() {
        let err = Error::string_conversion(OsString::from("wassup"));
        assert_eq!(err.to_string(), "cannot convert string to utf8: wassup")
    }

    #[test]
    fn uncallable_listener() {
        let err = Error::uncallable_listener("string");
        assert_eq!(err.to_string(), "string is not callable")
    }
}
