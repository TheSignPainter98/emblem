#[cfg(test)]
use crate::ast::AstDebug;
use core::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Text<'t> {
    Owned(String),
    Borrowed(&'t str),
}

impl<'t> Text<'t> {
    pub fn as_str(&'t self) -> &'t str {
        match self {
            Text::Owned(s) => s,
            Text::Borrowed(s) => s,
        }
    }
}

impl From<Text<'_>> for String {
    fn from(txt: Text<'_>) -> String {
        match txt {
            Text::Owned(s) => s,
            Text::Borrowed(s) => s.to_string(),
        }
    }
}

impl<'t> From<&'t str> for Text<'t> {
    fn from(s: &'t str) -> Self {
        Self::Borrowed(s)
    }
}

impl From<String> for Text<'_> {
    fn from(s: String) -> Self {
        Self::Owned(s)
    }
}

impl Display for Text<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

#[cfg(test)]
impl AstDebug for Text<'_> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        self.as_str().test_fmt(buf);
    }
}
