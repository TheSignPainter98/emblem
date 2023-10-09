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
    // use super::*;

    // TODO(kcza): make tests!
}
