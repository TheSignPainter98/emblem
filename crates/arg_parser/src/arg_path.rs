use crate::RawArgs;
use clap::{
    builder::{OsStr, StringValueParser, TypedValueParser},
    error::{Error as ClapError, ErrorKind as ClapErrorKind},
    CommandFactory,
};
use std::{fmt::Display, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgPath {
    Stdio,
    Path(PathBuf),
}

impl ArgPath {
    pub(crate) fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }
}

impl Default for ArgPath {
    fn default() -> Self {
        Self::Path("main.em".into())
    }
}

impl Display for ArgPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Stdio => "-",
                Self::Path(s) => s.to_str().unwrap_or("(invalid path)"),
            }
        )
    }
}

impl From<ArgPath> for emblem_core::ArgPath {
    fn from(path: ArgPath) -> Self {
        match path {
            ArgPath::Stdio => Self::Stdio,
            ArgPath::Path(p) => Self::Path(p),
        }
    }
}

impl TryFrom<OsStr> for ArgPath {
    type Error = ClapError;

    fn try_from(raw: OsStr) -> Result<Self, Self::Error> {
        if let Some(s) = raw.to_str() {
            return Self::try_from(s);
        }
        Err(RawArgs::command().error(
            ClapErrorKind::InvalidValue,
            format!("could not convert '{:?}' to a valid UTF-8 string", raw),
        ))
    }
}

impl TryFrom<String> for ArgPath {
    type Error = ClapError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl TryFrom<&str> for ArgPath {
    type Error = clap::Error;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        match raw {
            "" => {
                Err(RawArgs::command()
                    .error(ClapErrorKind::InvalidValue, FILE_PATH_CANNOT_BE_EMPTY))
            }
            "-" => Ok(Self::Stdio),
            raw => Ok(Self::Path(PathBuf::from(raw))),
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum UninferredArgPath {
    #[default]
    Infer,
    Stdio,
    Path(PathBuf),
}

impl UninferredArgPath {
    pub(crate) fn parser() -> impl TypedValueParser {
        StringValueParser::new().try_map(Self::try_from)
    }

    pub(crate) fn infer_from(&self, other: &ArgPath) -> ArgPath {
        match self {
            Self::Infer => match other {
                ArgPath::Stdio => ArgPath::Stdio,
                ArgPath::Path(s) => ArgPath::Path(s.clone()),
            },
            Self::Stdio => ArgPath::Stdio,
            Self::Path(s) => ArgPath::Path(s.clone()),
        }
    }
}

impl Display for UninferredArgPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Self::Infer => "??",
            Self::Stdio => "stdio",
            Self::Path(p) => p.to_str().unwrap(),
        };
        repr.fmt(f)
    }
}

impl TryFrom<OsStr> for UninferredArgPath {
    type Error = ClapError;

    fn try_from(raw: OsStr) -> Result<Self, Self::Error> {
        if let Some(s) = raw.to_str() {
            return Self::try_from(s);
        }
        Err(RawArgs::command().error(
            ClapErrorKind::InvalidValue,
            format!("could not convert '{:?}' to an OS string", raw),
        ))
    }
}

impl TryFrom<String> for UninferredArgPath {
    type Error = ClapError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::try_from(&raw[..])
    }
}

const FILE_PATH_CANNOT_BE_EMPTY: &str = "file path cannot be empty";

impl TryFrom<&str> for UninferredArgPath {
    type Error = ClapError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        match raw {
            "" => {
                Err(RawArgs::command()
                    .error(ClapErrorKind::InvalidValue, FILE_PATH_CANNOT_BE_EMPTY))
            }
            "-" => Ok(Self::Stdio),
            "??" => Ok(Self::Infer),
            path => Ok(Self::Path(path.into())),
        }
    }
}
