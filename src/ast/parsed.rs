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
    fn fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Call { name, args } => {
                buf.push(format!(".{}", name));
                if !args.is_empty() {
                    buf.push("(".into());
                    args.fmt(buf);
                    buf.push(")".into());
                }
            }
            Self::Word(w) => buf.push(format!("{:?}", w)),
            Self::Whitespace(w) => buf.push(format!("{:?}", w)),
            Self::Comment(c) => buf.push(format!("// {:?}", c)),
            Self::MultiLineComment(c) => AstDebug::fmt(c, buf),
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
    fn fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Word(w) => buf.push(format!("{:?}", w)),
            Self::Whitespace(w) => buf.push(format!("{:?}", w)),
            Self::Indented(i) => buf.push(format!("Indented({})", i)),
            Self::Nested(c) => {
                buf.push("/*".into());
                AstDebug::fmt(c, buf);
                buf.push("*/".into());
            }
        }
    }
}
