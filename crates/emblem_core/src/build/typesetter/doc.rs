use crate::{
    ast::{
        parsed::{Attr, Attrs, Content, ParsedFile, Sugar},
        Dash, Glue, Par, ParPart, ReprLoc,
    },
    parser::Location,
    FileContentSlice,
};

#[cfg(test)]
use crate::{ast::AstDebug, context::file_content::FileSlice};

pub type Doc = DocElem;

#[derive(Debug, Eq, PartialEq)]
pub enum DocElem {
    Word {
        word: FileContentSlice,
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
    Command {
        name: CommandName,
        plus: bool,
        attrs: Option<Attrs>,
        args: Vec<DocElem>,
        result: Option<Box<DocElem>>,
        loc: Location,
    },
    Content(Vec<DocElem>),
}

impl DocElem {
    fn into_content(self) -> Option<Vec<DocElem>> {
        match self {
            Self::Content(cs) => Some(cs),
            _ => None,
        }
    }

    fn simplify(self) -> Self {
        match self {
            Self::Content(c) if c.len() == 1 => c.into_iter().next().unwrap().simplify(),
            Self::Content(c) => Self::Content(c.into_iter().map(Self::simplify).collect()),
            Self::Command {
                name,
                plus,
                attrs,
                args,
                result,
                loc,
            } => Self::Command {
                name,
                plus,
                attrs,
                args: args.into_iter().map(Self::simplify).collect(),
                result,
                loc,
            },
            c => c,
        }
    }
}

impl Default for DocElem {
    fn default() -> Self {
        Self::Content(vec![])
    }
}

#[cfg(test)]
impl AstDebug for Doc {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Word { word, .. } => word.surround(buf, "Word(", ")"),
            Self::Dash { dash, .. } => dash.test_fmt(buf),
            Self::Glue { glue, .. } => glue.test_fmt(buf),
            Self::Command {
                name,
                plus,
                attrs,
                args,
                ..
            } => {
                ".".test_fmt(buf);
                name.test_fmt(buf);
                if *plus {
                    "+".test_fmt(buf);
                }
                if let Some(attrs) = attrs {
                    attrs.test_fmt(buf);
                }
                for arg in args {
                    arg.surround(buf, "{", "}");
                }
            }
            Self::Content(c) => c.test_fmt(buf),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommandName {
    Literal(&'static str),
    FileContentSlice(FileContentSlice),
}

impl From<&'static str> for CommandName {
    fn from(literal: &'static str) -> Self {
        Self::Literal(literal)
    }
}

impl From<FileContentSlice> for CommandName {
    fn from(slice: FileContentSlice) -> Self {
        Self::FileContentSlice(slice)
    }
}

#[cfg(test)]
impl AstDebug for CommandName {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        match self {
            Self::Literal(l) => l,
            Self::FileContentSlice(s) => s.to_str(),
        }
        .test_fmt(buf);
    }
}

#[derive(Clone)]
struct DocStackState {
    discern_pars: bool,
}

impl DocStackState {
    pub fn new() -> Self {
        Self { discern_pars: true }
    }

    pub fn with_discern_pars(&self, discern_pars: bool) -> Self {
        Self { discern_pars }
    }
}

impl From<ParsedFile> for Doc {
    fn from(parsed: ParsedFile) -> Self {
        parsed
            .into_doc(DocStackState::new())
            .unwrap_or_default()
            .simplify()
    }
}

trait IntoDoc {
    fn into_doc(self, state: DocStackState) -> Option<DocElem>;
}

impl IntoDoc for ParsedFile {
    fn into_doc(self, state: DocStackState) -> Option<DocElem> {
        self.pars.into_doc(state)
    }
}

impl IntoDoc for Vec<Par<ParPart<Content>>> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem> {
        let content: Vec<_> = self
            .into_iter()
            .flat_map(|par| {
                if par.is_empty() {
                    return None;
                }

                let loc = par.repr_loc();
                let converted = par
                    .into_doc(state.with_discern_pars(false))
                    .map(|d| match d {
                        DocElem::Content(cs) => DocElem::Content(
                            if cs.iter().all(|c| matches!(c, DocElem::Content(_))) {
                                cs.into_iter()
                                    .flat_map(|c| {
                                        c.into_content()
                                            .expect("internal error: content was not content")
                                    })
                                    .collect()
                            } else {
                                cs
                            },
                        ),
                        d => d,
                    });

                let apply_paragraph = state.discern_pars
                    && match &converted {
                        Some(DocElem::Content(c)) => {
                            !matches!(&c[..], [] | [DocElem::Command { .. }])
                        }
                        Some(DocElem::Command { .. }) => false,
                        _ => true,
                    };
                if apply_paragraph {
                    return Some(DocElem::Command {
                        name: "p".into(),
                        plus: false,
                        attrs: None,
                        result: None,
                        args: vec![converted.unwrap()],
                        loc,
                    });
                }
                converted
            })
            .collect();

        Some(if content.len() == 1 {
            content.into_iter().next().unwrap()
        } else {
            DocElem::Content(content)
        })
    }
}

impl IntoDoc for Par<ParPart<Content>> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem> {
        Some(DocElem::Content(
            self.parts
                .into_iter()
                .filter(|part| !part.is_empty())
                .filter_map(|part| part.into_doc(state.clone()))
                .collect(),
        ))
    }
}

impl IntoDoc for ParPart<Content> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem> {
        match self {
            Self::Line(l) => l.into_doc(state),
            Self::Command(c) => c.into_doc(state),
        }
    }
}

impl IntoDoc for Vec<Content> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem> {
        Some(DocElem::Content(
            self.into_iter()
                .filter_map(|c| c.into_doc(state.clone()))
                .collect(),
        ))
    }
}

impl IntoDoc for Content {
    fn into_doc(self, state: DocStackState) -> Option<DocElem> {
        match self {
            Self::Command {
                name,
                pluses,
                attrs,
                inline_args,
                remainder_arg,
                trailer_args,
                invocation_loc,
                ..
            } => Some(DocElem::Command {
                name: name.into(),
                plus: pluses != 0,
                attrs,
                args: {
                    inline_args
                        .into_iter()
                        .chain(remainder_arg)
                        .map(|arg| arg.into_doc(state.clone()).unwrap_or_default())
                        .chain(
                            trailer_args
                                .into_iter()
                                .flat_map(|arg| arg.into_doc(state.clone())),
                        )
                        .collect()
                },
                result: None,
                loc: invocation_loc,
            }),
            Self::Sugar(sugar) => sugar.into_doc(state),
            Self::Word { word, loc } => Some(DocElem::Word { word, loc }),
            Self::Dash { dash, loc } => Some(DocElem::Dash { dash, loc }),
            Self::Glue { glue, loc } => Some(DocElem::Glue { glue, loc }),
            Self::Verbatim { verbatim, loc } => Some(DocElem::Word {
                word: verbatim,
                loc,
            }),
            Self::Shebang { .. }
            | Self::Whitespace { .. }
            | Self::SpiltGlue { .. }
            | Self::Comment { .. }
            | Self::MultiLineComment { .. } => None,
        }
    }
}

impl IntoDoc for Sugar {
    fn into_doc(self, state: DocStackState) -> Option<DocElem> {
        Some({
            let name = self.call_name().into();
            let loc = self.repr_loc();

            match self {
                Self::Italic { arg, .. }
                | Self::Bold { arg, .. }
                | Self::Monospace { arg, .. }
                | Self::Smallcaps { arg, .. }
                | Self::AlternateFace { arg, .. } => DocElem::Command {
                    name,
                    plus: false,
                    attrs: None,
                    args: [arg.into_doc(state)].into_iter().flatten().collect(),
                    result: None,
                    loc,
                },
                Self::Heading { pluses, arg, .. } => DocElem::Command {
                    name,
                    plus: pluses != 0,
                    attrs: None,
                    args: [arg.into_doc(state)].into_iter().flatten().collect(),
                    result: None,
                    loc,
                },
                Self::Mark { mark, .. } => DocElem::Command {
                    name,
                    plus: false,
                    attrs: Some(Attrs::new(
                        vec![Attr::Unnamed {
                            value: mark.clone(),
                            raw: mark,
                            loc: loc.clone(),
                        }],
                        loc.clone(),
                    )),
                    args: vec![],
                    result: None,
                    loc,
                },
                Self::Reference { reference, .. } => DocElem::Command {
                    name,
                    plus: false,
                    attrs: Some(Attrs::new(
                        vec![Attr::Unnamed {
                            value: reference.clone(),
                            raw: reference,
                            loc: loc.clone(),
                        }],
                        loc.clone(),
                    )),
                    args: vec![],
                    result: None,
                    loc,
                },
            }
        })
    }
}

#[cfg(test)]
mod test {
    use crate::parser::test::ParserTest;

    #[test]
    fn into_doc() {
        ParserTest::new("empty").input("").produces_doc("[]");
        ParserTest::new("short")
            .input("foo")
            .produces_doc(".p{Word(foo)}");
        ParserTest::new("long").input(indoc::indoc!(
            r#"
                this is how to be a heart breaker
                _boys, they_ **like** `a` =little= ==danger==

                we'll get him !falling! for a stranger, a player
                singing---I lo-lo-love--you
                at~least~~I ~ think ~~ I ~do~~
            "#))
            .produces_doc("[.p{[Word(this)|Word(is)|Word(how)|Word(to)|Word(be)|Word(a)|Word(heart)|Word(breaker)|.it{[Word(boys,)|Word(they)]}|.bf{Word(like)}|.tt{Word(a)}|.sc{Word(little)}|.af{Word(danger)}]}|.p{[Word(we'll)|Word(get)|Word(him)|Word(falling)|Word(for)|Word(a)|Word(stranger,)|Word(a)|Word(player)|Word(singing)|---|Word(I)|Word(lo)|-|Word(lo)|-|Word(love)|--|Word(you)|Word(at)|~|Word(least)|~~|Word(I)|Word(think)|Word(I)|Word(do)]}]");
    }

    #[test]
    fn into_doc_headings() {
        ParserTest::new("headings")
            .input(indoc::indoc!(
                "
                    # h1

                    ## h2

                    ### h3

                    #### h4

                    ##### h5

                    ###### h6

                    #+ h1

                    ##+ h2

                    ###+ h3

                    ####+ h4

                    #####+ h5

                    ######+ h6
                "
            ))
            .produces_doc("[.h1{Word(h1)}|.h2{Word(h2)}|.h3{Word(h3)}|.h4{Word(h4)}|.h5{Word(h5)}|.h6{Word(h6)}|.h1+{Word(h1)}|.h2+{Word(h2)}|.h3+{Word(h3)}|.h4+{Word(h4)}|.h5+{Word(h5)}|.h6+{Word(h6)}]");
    }

    #[test]
    fn into_doc_commands() {
        ParserTest::new("inline-single")
            .input(".get{ready here we come}")
            .produces_doc(".get{[Word(ready)|Word(here)|Word(we)|Word(come)]}");
        ParserTest::new("inline-multi")
            .input(".its{time}{to have}: some fun")
            .produces_doc(".its{Word(time)}{[Word(to)|Word(have)]}{[Word(some)|Word(fun)]}");
        ParserTest::new("remainder-single")
            .input(".you: know the world is ours")
            .produces_doc(".you{[Word(know)|Word(the)|Word(world)|Word(is)|Word(ours)]}");
        ParserTest::new("trailer-single")
            .input(indoc::indoc!(
                "
                    .so:
                        now you're going down
                "
            ))
            .produces_doc(".so{[Word(now)|Word(you're)|Word(going)|Word(down)]}");
        ParserTest::new("multi-trailer-single")
            .input(indoc::indoc!(
                "
                    .so:
                        now you're going down
                "
            ))
            .produces_doc(".so{[Word(now)|Word(you're)|Word(going)|Word(down)]}");
        ParserTest::new("trailer-multi")
            .input(indoc::indoc!(
                "
                    .cos{that is}:
                        what
                    ::
                        we say
                ",
            ))
            .produces_doc(".cos{[Word(that)|Word(is)]}{Word(what)}{[Word(we)|Word(say)]}");
        ParserTest::new("p-single-pars")
            .input(indoc::indoc!(
                "
                    .p:
                        we are

                    .p:
                        operation doomsday
                "
            ))
            .produces_doc("[.p{[Word(we)|Word(are)]}|.p{[Word(operation)|Word(doomsday)]}]");
        ParserTest::new("p-multi-in-par")
            .input(indoc::indoc!(
                "
                    .p:
                        we are
                    .p:
                        operation doomsday
                "
            ))
            .produces_doc(".p{[.p{[Word(we)|Word(are)]}|.p{[Word(operation)|Word(doomsday)]}]}");
        ParserTest::new("empty-preserved")
            .input(indoc::indoc!(
                "
                    .we{}{are}{}{operation}{}{doomsday}{}
                "
            ))
            .produces_doc(".we{[]}{Word(are)}{[]}{Word(operation)}{[]}{Word(doomsday)}{[]}");
    }

    #[test]
    fn into_doc_comments() {
        ParserTest::new("line-comment")
            .input("// on this final night")
            .produces_doc("[]");
        ParserTest::new("line-comment-in-arg")
            .input(".it: // on this final night")
            .produces_doc(".it{[]}");
        ParserTest::new("line-comment-inline")
            .input("before you // disappear")
            .produces_doc(".p{[Word(before)|Word(you)]}");
        ParserTest::new("nested-comment")
            .input("/* forget all your worries */")
            .produces_doc("[]");
        ParserTest::new("nested-comment-inline")
            .input("and let go of /* all */ your fears")
            .produces_doc(".p{[Word(and)|Word(let)|Word(go)|Word(of)|Word(your)|Word(fears)]}");
    }
}
