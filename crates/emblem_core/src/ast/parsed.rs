use crate::ast::{Dash, File, Glue, Par, ParPart};
use crate::context::file_content::FileSlice;
use crate::parser::Location;
use crate::FileContentSlice;

#[cfg(test)]
use crate::ast::AstDebug;

pub type ParsedFile = File<ParPart<Content>>;

#[allow(clippy::large_enum_variant)] // TODO(kcza): re-evaluate this (requires benchmarks)
#[derive(Debug)]
pub enum Content {
    Shebang {
        text: FileContentSlice,
        loc: Location,
    },
    Command {
        qualifier: Option<FileContentSlice>,
        name: FileContentSlice,
        pluses: usize,
        attrs: Option<Attrs>,
        inline_args: Vec<Vec<Content>>,
        remainder_arg: Option<Vec<Content>>,
        trailer_args: Vec<Vec<Par<ParPart<Content>>>>,
        loc: Location,
        invocation_loc: Location,
    },
    Sugar(Sugar),
    Word {
        word: FileContentSlice,
        loc: Location,
    },
    Whitespace {
        whitespace: FileContentSlice,
        loc: Location,
    },
    Dash {
        dash: Dash,
        loc: Location,
    },
    Glue {
        glue: Glue,
        loc: Location,
    },
    SpiltGlue {
        raw: FileContentSlice,
        loc: Location,
    },
    Verbatim {
        verbatim: FileContentSlice,
        loc: Location,
    },
    Comment {
        comment: FileContentSlice,
        loc: Location,
    },
    MultiLineComment {
        content: MultiLineComment,
        loc: Location,
    },
}

#[cfg(test)]
impl AstDebug for Content {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Shebang { text, .. } => text.surround(buf, "Shebang(", ")"),
            Self::Command {
                qualifier,
                name,
                pluses,
                attrs,
                inline_args,
                remainder_arg,
                trailer_args,
                ..
            } => {
                buf.push('.'.into());
                if let Some(qualifier) = qualifier {
                    qualifier.surround(buf, "(", ")");
                    buf.push('.'.into());
                }
                name.test_fmt(buf);
                if let Some(attrs) = attrs {
                    attrs.test_fmt(buf);
                }
                if *pluses > 0 {
                    "+".repeat(*pluses).surround(buf, "(", ")")
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
            Self::Sugar(s) => s.test_fmt(buf),
            Self::Word { word, .. } => word.surround(buf, "Word(", ")"),
            Self::Whitespace { whitespace, .. } => whitespace.surround(buf, "<", ">"),
            Self::Dash { dash, .. } => dash.test_fmt(buf),
            Self::Glue { glue, .. } => glue.test_fmt(buf),
            Self::SpiltGlue { raw, .. } => raw.surround(buf, "SpiltGlue(", ")"),
            Self::Verbatim { verbatim, .. } => verbatim.surround(buf, "!", "!"),
            Self::Comment { comment, .. } => {
                buf.push("//".into());
                comment.test_fmt(buf);
            }
            Self::MultiLineComment { content, .. } => content.surround(buf, "/*", "*/"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Attrs {
    attrs: Vec<Attr>,
    loc: Location,
}

impl Attrs {
    pub fn new(attrs: Vec<Attr>, loc: Location) -> Self {
        Self { attrs, loc }
    }

    pub fn args(&self) -> &[Attr] {
        &self.attrs
    }

    pub fn loc(&self) -> &Location {
        &self.loc
    }
}

#[cfg(test)]
impl AstDebug for Attrs {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        self.args().test_fmt(buf);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attr {
    // TODO(kcza): make near-duplicate to allow preservation to the appropriate point.
    Named {
        name: FileContentSlice,
        value: FileContentSlice,
        raw: FileContentSlice,
        loc: Location,
    },
    Unnamed {
        value: FileContentSlice,
        raw: FileContentSlice,
        loc: Location,
    },
}

impl Attr {
    pub fn named(raw: FileContentSlice, loc: Location) -> Self {
        let eq_idx = raw
            .to_str()
            .find('=')
            .expect("internal error: named attribute had no equals sign");
        let name = raw.slice(..eq_idx).trim();
        let value = raw.slice(eq_idx + 1..).trim();
        Self::Named {
            name,
            value,
            raw,
            loc,
        }
    }

    pub fn unnamed(raw: FileContentSlice, loc: Location) -> Self {
        let value = raw.trimmed();
        Self::Unnamed { raw, value, loc }
    }

    pub fn name(&self) -> Option<&FileContentSlice> {
        match self {
            Self::Named { name, .. } => Some(name),
            Self::Unnamed { .. } => None,
        }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> &FileContentSlice {
        match self {
            Self::Named { value, .. } | Self::Unnamed { value, .. } => value,
        }
    }

    pub fn repr(&self) -> &FileContentSlice {
        match self {
            Self::Named { name, .. } => name,
            Self::Unnamed { value, .. } => value,
        }
    }

    pub fn loc(&self) -> &Location {
        match self {
            Self::Named { loc, .. } => loc,
            Self::Unnamed { loc, .. } => loc,
        }
    }

    #[allow(dead_code)]
    fn raw(&self) -> &FileContentSlice {
        match self {
            Self::Named { raw, .. } => raw,
            Self::Unnamed { raw, .. } => raw,
        }
    }
}

#[cfg(test)]
impl AstDebug for Attr {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Unnamed { value, .. } => {
                value.surround(buf, "(", ")");
            }
            Self::Named { name, value, .. } => {
                name.surround(buf, "(", ")");
                buf.push("=".into());
                value.surround(buf, "(", ")");
            }
        }
    }
}

#[derive(Debug)]
pub enum Sugar {
    Italic {
        delimiter: FileContentSlice,
        arg: Vec<Content>,
        loc: Location,
    },
    Bold {
        delimiter: FileContentSlice,
        arg: Vec<Content>,
        loc: Location,
    },
    Monospace {
        arg: Vec<Content>,
        loc: Location,
    },
    Smallcaps {
        arg: Vec<Content>,
        loc: Location,
    },
    AlternateFace {
        arg: Vec<Content>,
        loc: Location,
    },
    Heading {
        level: usize,
        pluses: usize,
        standoff: FileContentSlice,
        arg: Vec<Content>,
        loc: Location,
        invocation_loc: Location,
    },
    Mark {
        mark: FileContentSlice,
        loc: Location,
    },
    Reference {
        reference: FileContentSlice,
        loc: Location,
    },
}

impl Sugar {
    pub fn call_name(&self) -> &'static str {
        match self {
            Self::Italic { .. } => "it",
            Self::Bold { .. } => "bf",
            Self::Monospace { .. } => "tt",
            Self::Smallcaps { .. } => "sc",
            Self::AlternateFace { .. } => "af",
            Self::Heading { level, .. } => match level {
                1 => "h1",
                2 => "h2",
                3 => "h3",
                4 => "h4",
                5 => "h5",
                6 => "h6",
                _ => panic!("internal error: unknown heading level {level}"),
            },
            Self::Mark { .. } => "mark",
            Self::Reference { .. } => "ref",
        }
    }
}

#[cfg(test)]
impl AstDebug for Sugar {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        buf.push(format!("${}", self.call_name()));
        match self {
            Self::Italic { arg, delimiter, .. } => {
                delimiter.surround(buf, "(", ")");
                arg.surround(buf, "{", "}");
            }
            Self::Bold { arg, delimiter, .. } => {
                delimiter.surround(buf, "(", ")");
                arg.surround(buf, "{", "}");
            }
            Self::Monospace { arg, .. } => {
                arg.surround(buf, "{", "}");
            }
            Self::Smallcaps { arg, .. } => {
                arg.surround(buf, "{", "}");
            }
            Self::AlternateFace { arg, .. } => {
                arg.surround(buf, "{", "}");
            }
            Self::Heading { arg, pluses, .. } => {
                if *pluses > 0 {
                    "+".repeat(*pluses).surround(buf, "(", ")");
                }
                arg.surround(buf, "{", "}");
            }
            Self::Mark { mark, .. } => {
                mark.surround(buf, "[", "]");
            }
            Self::Reference { reference, .. } => {
                reference.surround(buf, "[", "]");
            }
        }
    }
}

#[derive(Debug)]
pub struct MultiLineComment(pub Vec<MultiLineCommentPart>);

#[cfg(test)]
impl AstDebug for MultiLineComment {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        self.0.test_fmt(buf);
    }
}

#[derive(Debug)]
pub enum MultiLineCommentPart {
    Newline,
    Comment(FileContentSlice),
    Nested(MultiLineComment),
}

#[cfg(test)]
impl AstDebug for MultiLineCommentPart {
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
    use crate::parser::Point;

    mod attrs {
        use super::*;
        use crate::Context;

        #[test]
        fn args() {
            let ctx = Context::new();
            let p1 = Point::at_start_of(
                ctx.alloc_file_name("fname.em"),
                ctx.alloc_file_content("helloworld"),
            );
            let p2 = p1.clone().shift("hello");
            let p3 = p2.clone().shift("world");
            let tests = vec![
                vec![],
                vec![
                    Attr::unnamed(p2.src().clone(), Location::new(&p1, &p2)),
                    Attr::unnamed(p3.src().clone(), Location::new(&p2, &p3)),
                ],
            ];

            for test in tests {
                assert_eq!(
                    Attrs::new(test.clone(), Location::new(&p1, &p2)).args(),
                    &test
                );
            }
        }
    }

    mod attr {
        use super::*;
        use crate::Context;

        #[test]
        fn unnamed() {
            let ctx = Context::new();
            let raw = " \tfoo\t ";
            let p1 =
                Point::at_start_of(ctx.alloc_file_name("fname.em"), ctx.alloc_file_content(raw));
            let attr = Attr::unnamed(p1.src().clone(), Location::new(&p1, &p1.clone().shift(raw)));

            assert_eq!(attr.name(), None);
            assert_eq!(attr.repr(), "foo");
            assert_eq!(attr.value(), "foo");
            assert_eq!(attr.raw(), raw);
        }

        #[test]
        fn named() {
            let ctx = Context::new();
            let raw = " \tfoo\t =\t bar \t";
            let p1 =
                Point::at_start_of(ctx.alloc_file_name("fname.em"), ctx.alloc_file_content(raw));
            let attr = Attr::named(p1.src().clone(), Location::new(&p1, &p1.clone().shift(raw)));

            assert_eq!(attr.name().unwrap(), "foo");
            assert_eq!(attr.repr(), "foo");
            assert_eq!(attr.value(), "bar");
            assert_eq!(attr.raw(), raw);
        }
    }

    mod sugar {
        use super::*;
        use crate::Context;

        #[test]
        fn call_name() {
            let ctx = Context::new();
            let text = "hello, world!";
            let p1 =
                Point::at_start_of(ctx.alloc_file_name("main.em"), ctx.alloc_file_content(text));
            let p2 = p1.clone().shift(text);
            let loc = Location::new(&p1, &p2);

            assert_eq!(
                "it",
                Sugar::Italic {
                    delimiter: ctx.alloc_file_content("_").into(),
                    arg: vec![],
                    loc: loc.clone()
                }
                .call_name()
            );
            assert_eq!(
                "bf",
                Sugar::Bold {
                    delimiter: ctx.alloc_file_content("**").into(),
                    arg: vec![],
                    loc: loc.clone()
                }
                .call_name()
            );
            assert_eq!(
                "tt",
                Sugar::Monospace {
                    arg: vec![],
                    loc: loc.clone()
                }
                .call_name()
            );
            assert_eq!(
                "sc",
                Sugar::Smallcaps {
                    arg: vec![],
                    loc: loc.clone()
                }
                .call_name()
            );
            assert_eq!(
                "af",
                Sugar::AlternateFace {
                    arg: vec![],
                    loc: loc.clone()
                }
                .call_name()
            );

            let standoff = ctx.alloc_file_content(" ");
            for level in 1..=6 {
                for pluses in 0..=2 {
                    assert_eq!(
                        format!("h{level}"),
                        Sugar::Heading {
                            level,
                            pluses,
                            standoff: standoff.clone().into(),
                            arg: vec![],
                            loc: loc.clone(),
                            invocation_loc: loc.clone(),
                        }
                        .call_name()
                    );
                }
            }
        }
    }
}
