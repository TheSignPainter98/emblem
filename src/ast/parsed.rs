// use std::fmt::Display;

use crate::ast::{text::Text, Line};

#[cfg(test)]
use crate::ast::AstDebug;

#[derive(Debug)]
pub enum Content<'i> {
    Call {
        name: Text<'i>,
        args: Vec<Line<Content<'i>>>,
    },
    Word(Text<'i>),
    Whitespace(&'i str),
    Comment(&'i str),
    MultiLineComment(MultiLineComment<'i>),
}

// impl Display for Content<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Call { name, args } => {
//                 write!(f, "[.{}", name)?;
//                 for arg in args {
//                     write!(f, "{{{}}}", arg)?;
//                 }
//                 write!(f, "]")
//             }
//             Self::Word(w) => w.fmt(f),
//             Self::Whitespace(w) => write!(f, "{:?}", w),
//             Self::Comment(c) => write!(f, "[// {:?}]", c),
//             Self::MultiLineComment(c) => write!(f, "/*{}*/", c),
//         }
//     }
// }

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
            Self::Whitespace(w) => w.surround(buf, "<", ">"),
            Self::Comment(c) => {
                buf.push("//".into());
                c.test_fmt(buf);
            }
            Self::MultiLineComment(c) => c.surround(buf, "/*", "*/"),
        }
    }
}

#[derive(Debug)]
pub struct MultiLineComment<'i>(pub Vec<MultiLineCommentPart<'i>>);

// impl Display for MultiLineComment<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for c in &self.0 {
//             c.fmt(f)?;
//         }
//         Ok(())
//     }
// }

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

// impl Display for MultiLineCommentPart<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Newline => write!(f, r"\n"),
//             Self::Comment(w) => write!(f, "{:?}", w),
//             Self::Nested(c) => write!(f, "/*{}*/", c),
//         }
//     }
// }

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
