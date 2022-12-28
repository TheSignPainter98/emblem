use std::fmt::Display;

use crate::ast::text::Text;
#[cfg(test)]
use crate::ast::AstDebug;

#[derive(Debug)]
pub enum Content<'i> {
    Call {
        name: Text<'i>,
        args: Vec<Content<'i>>,
    },
    Word(Text<'i>),
    Whitespace(&'i str),
    Comment(&'i str),
    MultiLineComment(MultiLineComment<'i>),
}

impl Display for Content<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Call { name, args } => {
                write!(f, "[.{}", name)?;
                for arg in args {
                    write!(f, "{{{}}}", arg)?;
                }
                write!(f, "]")
            }
            Self::Word(w) => w.fmt(f),
            Self::Whitespace(w) => write!(f, "{:?}", w),
            Self::Comment(c) => write!(f, "[// {:?}]", c),
            Self::MultiLineComment(c) => write!(f, "/*{}*/", c),
        }
    }
}

#[cfg(test)]
impl AstDebug for Content<'_> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Call { name, args } => {
                buf.push('.'.into());
                name.test_fmt(buf);
                args.test_fmt(buf);
            }
            Self::Word(w) => w.surround(buf, "Word(", ")"),
            Self::Whitespace(w) => w.surround(buf, "W(", ")"),
            Self::Comment(c) => {
                buf.push("//".into());
                c.test_fmt(buf);
            }
            Self::MultiLineComment(c) => c.test_fmt(buf),
        }
    }
}

#[derive(Debug)]
pub enum MultiLineComment<'i> {
    Word(&'i str),
    Whitespace(&'i str),
    Indented(Box<MultiLineComment<'i>>),
    Nested(Box<MultiLineComment<'i>>),
}

impl Display for MultiLineComment<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Word(w) => write!(f, "[{}]", w),
            Self::Whitespace(w) => write!(f, "[{}]", w),
            Self::Indented(i) => write!(f, "Indented({})", i),
            Self::Nested(c) => write!(f, "/*{}*/", c),
        }
    }
}

#[cfg(test)]
impl AstDebug for MultiLineComment<'_> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Word(w) => w.test_fmt(buf),
            Self::Whitespace(w) => w.test_fmt(buf),
            Self::Indented(i) => i.surround(buf, "Indented(", ")"),
            Self::Nested(c) => c.surround(buf, "/*", "*/"),
        }
    }
}
