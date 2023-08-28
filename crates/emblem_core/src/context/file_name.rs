use std::{fmt::Display, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileName {
    inner: Rc<str>,
}

impl FileName {
    pub(super) fn new(raw: &str) -> Self {
        Self {
            inner: Rc::from(raw),
        }
    }
}

impl Default for FileName {
    fn default() -> Self {
        Self::new("")
    }
}

impl AsRef<str> for FileName {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}

impl PartialEq<&str> for FileName {
    fn eq(&self, other: &&str) -> bool {
        self.inner.as_ref() == *other
    }
}

impl PartialEq<FileName> for &str {
    fn eq(&self, other: &FileName) -> bool {
        *self == other.as_ref()
    }
}

impl PartialEq<&str> for &FileName {
    fn eq(&self, other: &&str) -> bool {
        self.inner.as_ref() == *other
    }
}

impl PartialEq<&FileName> for &str {
    fn eq(&self, other: &&FileName) -> bool {
        *self == other.as_ref()
    }
}

impl Display for FileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
