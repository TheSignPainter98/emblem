use core::fmt::{self, Display, Formatter};

#[derive(Debug)]
enum Text<'t> {
    Owned(String),
    Borrowed(&'t str),
}

impl AsRef<str> for Text<'_> {
    fn as_ref(&self) -> &str {
        match self {
            Text::Owned(s) => &s,
            Text::Borrowed(s) => s,
        }
    }
}

impl Into<String> for Text<'_> {
    fn into(self) -> String {
        match self {
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
        self.as_ref().fmt(f)
    }
}
