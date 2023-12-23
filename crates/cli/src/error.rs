use std::{borrow::Cow, error::Error as StdError, fmt::Display, io};

use camino::Utf8PathBuf;
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

    pub fn io(path: impl Into<Utf8PathBuf>, cause: io::Error) -> Self {
        let path = path.into();
        Self::new(ErrorImpl::IO { path, cause })
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
}

impl<T: Into<ErrorImpl>> From<T> for Error {
    fn from(cause: T) -> Self {
        Self::new(cause.into())
    }
}

impl From<Error> for Log {
    fn from(error: Error) -> Self {
        (*error.0).into()
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

    #[error("IO error accessing {path}: {cause}")]
    IO { path: Utf8PathBuf, cause: io::Error },

    #[error("manifest invalid: {reason}")]
    ManifestInvalid { reason: Cow<'static, str> },

    #[error("{context}: {cause}")]
    WithContext {
        context: Cow<'static, str>,
        cause: Error,
    },

    #[error("toml deserialisation error: {0}")]
    TomlDeserialisation(#[from] toml_edit::de::Error),
}

impl From<ErrorImpl> for Log {
    fn from(error: ErrorImpl) -> Self {
        use ErrorImpl::*;
        match error {
            ArgInvalid { arg, reason } => {
                Log::error(reason).add_info(format!("in argument '{arg}'"))
            }
            EmblemCore(e) => e.into(),
            Git(e) => Log::error("git error").add_info(e.to_string()),
            IO { path, cause } => {
                Log::error(format!("cannot access {path}")).add_info(cause.to_string())
            }
            ManifestInvalid { reason } => {
                Log::error("manifest invalid").add_info(reason.to_string())
            }
            WithContext { context, cause } => Log::from(cause).add_info(context.to_string()),
            TomlDeserialisation(e) => {
                Log::error(format!("cannot parse toml")).add_info(e.to_string())
            }
        }
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
    fn io() {
        assert_eq!(
            Error::io(
                Utf8PathBuf::from("file.em"),
                io::Error::new(io::ErrorKind::BrokenPipe, "oh no!")
            )
            .to_string(),
            "IO error accessing file.em: oh no!"
        );
    }

    #[test]
    fn manifest_invalid() {
        assert_eq!(
            Error::manifest_invalid("lmao").to_string(),
            "manifest invalid: lmao"
        )
    }

    #[test]
    fn log_consistency() {
        let core_error = emblem_core::Error::io(
            "/dev/null",
            io::Error::new(io::ErrorKind::BrokenPipe, "oh no!"),
        );
        let cli_error = Error::io(
            "/dev/null",
            io::Error::new(io::ErrorKind::BrokenPipe, "oh no!"),
        );
        assert_eq!(Log::from(core_error), Log::from(cli_error));
    }
}
