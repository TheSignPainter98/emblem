use std::{borrow::Cow, error::Error as StdError, fmt::Display, io};

use emblem_core::Log;

#[derive(Debug)]
pub struct Error(Box<ErrorImpl>);

impl Error {
    fn new(error: ErrorImpl) -> Self {
        Self(Box::new(error))
    }

    pub fn arg_invalid(arg: String, reason: impl Into<Cow<'static, str>>) -> Self {
        let reason = reason.into();
        Self::new(ErrorImpl::ArgInvalid { arg, reason })
    }

    pub fn context(self, context: impl Into<Cow<'static, str>>) -> Self {
        let context = context.into();
        Self::new(ErrorImpl::WithContext {
            context,
            cause: self,
        })
    }

    pub fn manifest_invalid(reason: impl Into<Cow<'static, str>>) -> Self {
        let reason = reason.into();
        Self::new(ErrorImpl::ManifestInvalid { reason })
    }

    pub fn unused_args(args: Vec<String>) -> Self {
        Self::new(ErrorImpl::UnusedArgs(args))
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

#[derive(Debug, thiserror::Error)]
enum ErrorImpl {
    #[error("argument '{arg}' invalid: {reason}")]
    ArgInvalid {
        arg: String,
        reason: Cow<'static, str>,
    },

    #[error("{0}")]
    EmblemCore(#[from] emblem_core::Error),

    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("manifest invalid: {reason}")]
    ManifestInvalid { reason: Cow<'static, str> },

    #[error("unused arguments: {}", .0.join(", "))]
    UnusedArgs(Vec<String>),

    #[error("{context}: {cause}")]
    WithContext {
        context: Cow<'static, str>,
        cause: Error,
    },

    #[error("yaml conversion error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

impl From<Error> for Log {
    fn from(_error: Error) -> Self {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arg_invalid() {
        assert_eq!(
            Error::arg_invalid("foo".into(), ":kekw:").to_string(),
            "argument 'foo' invalid: :kekw:"
        )
    }

    #[test]
    fn manifest_invalid() {
        assert_eq!(
            Error::manifest_invalid("lmao").to_string(),
            "manifest invalid: lmao"
        )
    }

    #[test]
    fn unused_args() {
        assert_eq!(
            Error::unused_args(
                ["hello", "world"]
                    .into_iter()
                    .map(ToOwned::to_owned)
                    .collect()
            )
            .to_string(),
            "unused arguments: hello, world"
        );
    }
}
