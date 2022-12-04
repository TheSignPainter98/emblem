
#[derive(Debug)]
enum Text<'t> {
    Owned(String),
    Borrowed(&'t str),
}

impl<'t> AsRef<str> for Text<'t> {
    fn as_ref(&self) -> &str {
        match self {
            Text::Owned(s) => &s,
            Text::Borrowed(s) => s,
        }
    }
}

impl<'t> Into<String> for Text<'t> {
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
