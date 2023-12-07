use std::fmt;

use camino::Utf8PathBuf;

#[cfg(test)]
use camino::Utf8Path;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ArgPath {
    Stdio,
    Path(Utf8PathBuf),
}

impl AsRef<ArgPath> for ArgPath {
    fn as_ref(&self) -> &ArgPath {
        self
    }
}

impl fmt::Display for ArgPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdio => write!(f, "-"),
            Self::Path(p) => write!(f, "{p}"),
        }
    }
}

#[cfg(test)]
impl ArgPath {
    pub fn path(&self) -> Option<&Utf8Path> {
        match self {
            Self::Stdio => None,
            Self::Path(p) => Some(p),
        }
    }
}
