use crate::ast::{text::Text, Dash, File, Glue, Par, ParPart};
use crate::parser::Location;

#[cfg(test)]
use crate::ast::AstDebug;

pub type ParsedFile<'i> = File<ParPart<Content<'i>>>;

#[derive(Debug)]
pub enum Content<'i> {
    Command {
        name: Text<'i>,
        attrs: Option<Attrs<'i>>,
        inline_args: Vec<Vec<Content<'i>>>,
        remainder_arg: Option<Vec<Content<'i>>>,
        trailer_args: Vec<Vec<Par<ParPart<Content<'i>>>>>,
        loc: Location<'i>,
    },
    Word {
        word: Text<'i>,
        loc: Location<'i>,
    },
    Whitespace {
        whitespace: &'i str,
        loc: Location<'i>,
    },
    Dash {
        dash: Dash,
        loc: Location<'i>,
    },
    Glue {
        glue: Glue,
        loc: Location<'i>,
    },
    Verbatim {
        verbatim: &'i str,
        loc: Location<'i>,
    },
    Comment {
        comment: &'i str,
        loc: Location<'i>,
    },
    MultiLineComment {
        content: MultiLineComment<'i>,
        loc: Location<'i>,
    },
}

#[cfg(test)]
impl AstDebug for Content<'_> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Command {
                name,
                attrs,
                inline_args,
                remainder_arg,
                trailer_args,
                ..
            } => {
                buf.push('.'.into());
                name.test_fmt(buf);
                if let Some(attrs) = attrs {
                    attrs.test_fmt(buf);
                }
                for arg in inline_args.iter() {
                    arg.surround(buf, "{", "}");
                }
                if let Some(arg) = remainder_arg {
                    buf.push(":".into());
                    arg.test_fmt(buf)
                }
                for arg in trailer_args.iter() {
                    buf.push("::".into());
                    arg.test_fmt(buf);
                }
            }
            Self::Word { word, .. } => word.surround(buf, "Word(", ")"),
            Self::Whitespace { whitespace, .. } => whitespace.surround(buf, "<", ">"),
            Self::Dash { dash, .. } => dash.test_fmt(buf),
            Self::Glue { glue, .. } => glue.test_fmt(buf),
            Self::Verbatim { verbatim, .. } => verbatim.surround(buf, "!", "!"),
            Self::Comment { comment, .. } => {
                buf.push("//".into());
                comment.test_fmt(buf);
            }
            Self::MultiLineComment { content, .. } => content.surround(buf, "/*", "*/"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attrs<'i>(Vec<Attr<'i>>);

impl<'i> Attrs<'i> {
    pub fn new(attrs: Vec<Attr<'i>>) -> Self {
        Self(attrs)
    }

    #[allow(dead_code)]
    pub fn args(&self) -> &Vec<Attr<'i>> {
        &self.0
    }
}

#[cfg(test)]
impl AstDebug for Attrs<'_> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        self.0.test_fmt(buf);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attr<'i> {
    Named { eq_idx: usize, raw: &'i str },
    Unnamed { raw: &'i str },
}

impl<'i> Attr<'i> {
    pub fn named(raw: &'i str) -> Self {
        Self::Named {
            eq_idx: raw.find('=').unwrap(),
            raw,
        }
    }

    pub fn unnamed(raw: &'i str) -> Self {
        Self::Unnamed { raw }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            Self::Named { raw, eq_idx } => raw[..*eq_idx].trim(),
            Self::Unnamed { raw } => raw.trim(),
        }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> Option<&str> {
        match self {
            Self::Named { raw, eq_idx } => Some(raw[eq_idx + 1..].trim()),
            Self::Unnamed { .. } => None,
        }
    }

    #[allow(dead_code)]
    fn raw(&self) -> &str {
        match self {
            Self::Named { raw, .. } => raw,
            Self::Unnamed { raw, .. } => raw,
        }
    }
}

#[cfg(test)]
impl AstDebug for Attr<'_> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Unnamed { .. } => {
                self.raw().surround(buf, "(", ")");
            }
            Self::Named { eq_idx, .. } => {
                let raw = self.raw();
                (&raw[..*eq_idx]).surround(buf, "(", ")");
                buf.push("=".into());
                (&raw[*eq_idx + 1..]).surround(buf, "(", ")");
            }
        }
    }
}

#[derive(Debug)]
pub struct MultiLineComment<'i>(pub Vec<MultiLineCommentPart<'i>>);

#[cfg(test)]
impl AstDebug for MultiLineComment<'_> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        self.0.test_fmt(buf);
    }
}

#[derive(Debug)]
pub enum MultiLineCommentPart<'i> {
    Newline,
    Comment(&'i str),
    Nested(MultiLineComment<'i>),
}

#[cfg(test)]
impl AstDebug for MultiLineCommentPart<'_> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Newline => buf.push(r"\n".into()),
            Self::Comment(w) => w.test_fmt(buf),
            Self::Nested(c) => {
                buf.push("Nested".into());
                c.test_fmt(buf);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod attrs {
        use super::*;

        #[test]
        fn args() {
            let tests = vec![vec![], vec![Attr::unnamed("hello"), Attr::unnamed("world")]];

            for test in tests {
                assert_eq!(Attrs::new(test.clone()).args(), &test);
            }
        }
    }

    mod attr {
        use super::*;

        #[test]
        fn unnamed() {
            let raw = " \tfoo\t ";
            let attr = Attr::unnamed(raw);

            assert_eq!(attr.name(), "foo");
            assert_eq!(attr.value(), None);
            assert_eq!(attr.raw(), raw);
        }

        #[test]
        fn named() {
            let raw = " \tfoo\t =\t bar \t";
            let attr = Attr::named(raw);

            assert_eq!(attr.name(), "foo");
            assert_eq!(attr.value(), Some("bar"));
            assert_eq!(attr.raw(), raw);
        }
    }
}
