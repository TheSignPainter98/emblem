use crate::ast::{text::Text, Dash, File, Glue, Par, ParPart};

#[cfg(test)]
use crate::ast::AstDebug;

pub type ParsedFile<'i> = File<ParPart<Content<'i>>>;

#[derive(Debug)]
pub enum Content<'i> {
    Command {
        name: Text<'i>,
        inline_args: Vec<Vec<Content<'i>>>,
        remainder_arg: Option<Vec<Content<'i>>>,
        trailer_args: Vec<Vec<Par<ParPart<Content<'i>>>>>,
    },
    Word(Text<'i>),
    Whitespace(&'i str),
    Dash(Dash),
    Glue(Glue),
    Verbatim(&'i str),
    Comment(&'i str),
    MultiLineComment(MultiLineComment<'i>),
}

#[cfg(test)]
impl AstDebug for Content<'_> {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Command {
                name,
                inline_args,
                remainder_arg,
                trailer_args: trailing_args,
            } => {
                buf.push('.'.into());
                name.test_fmt(buf);
                for arg in inline_args.iter() {
                    arg.surround(buf, "{", "}");
                }
                if let Some(arg) = remainder_arg {
                    buf.push(":".into());
                    arg.test_fmt(buf)
                }
                for arg in trailing_args.iter() {
                    buf.push("::".into());
                    arg.test_fmt(buf);
                }
            }
            Self::Word(w) => w.surround(buf, "Word(", ")"),
            Self::Whitespace(w) => w.surround(buf, "<", ">"),
            Self::Dash(d) => d.test_fmt(buf),
            Self::Glue(g) => g.test_fmt(buf),
            Self::Verbatim(v) => v.surround(buf, "!", "!"),
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
