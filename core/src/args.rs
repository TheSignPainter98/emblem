use std::{fmt, path};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgPath {
    Stdio,
    Path(path::PathBuf),
}

impl AsRef<ArgPath> for ArgPath {
    fn as_ref(&self) -> &ArgPath {
        self
    }
}

impl fmt::Display for ArgPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[cfg(test)]
impl ArgPath {
    pub fn path(&self) -> Option<&path::Path> {
        match self {
            Self::Stdio => None,
            Self::Path(p) => Some(p),
        }
    }
}
