use crate::{
    ast::{
        parsed::{Attr, Attrs, Content, ParsedFile, Sugar},
        Dash, Glue, Par, ParPart, ReprLoc, Text,
    },
    parser::Location,
};

#[cfg(test)]
use crate::ast::AstDebug;

pub type Doc<'em> = DocElem<'em>;

#[derive(Debug, Eq, PartialEq)]
pub enum DocElem<'em> {
    Word {
        word: Text<'em>,
        loc: Location<'em>,
    },
    Dash {
        dash: Dash,
        loc: Location<'em>,
    },
    Glue {
        glue: Glue,
        loc: Location<'em>,
    },
    Command {
        name: Text<'em>,
        plus: bool,
        attrs: Option<Attrs<'em>>,
        args: Vec<DocElem<'em>>,
        result: Option<Box<DocElem<'em>>>,
        loc: Location<'em>,
    },
    Content(Vec<DocElem<'em>>),
}

impl<'em> DocElem<'em> {
    fn into_content(self) -> Option<Vec<DocElem<'em>>> {
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

impl Default for DocElem<'_> {
    fn default() -> Self {
        Self::Content(vec![])
    }
}

#[cfg(test)]
impl<'em> AstDebug for Doc<'em> {
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

impl<'i> From<ParsedFile<'i>> for Doc<'i> {
    fn from(parsed: ParsedFile<'i>) -> Self {
        parsed
            .into_doc(DocStackState::new())
            .unwrap_or_default()
            .simplify()
    }
}

trait IntoDoc<'em> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem<'em>>;
}

impl<'em> IntoDoc<'em> for ParsedFile<'em> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem<'em>> {
        self.pars.into_doc(state)
    }
}

impl<'em> IntoDoc<'em> for Vec<Par<ParPart<Content<'em>>>> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem<'em>> {
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
                        name: Text::from("p"),
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

impl<'em> IntoDoc<'em> for Par<ParPart<Content<'em>>> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem<'em>> {
        Some(DocElem::Content(
            self.parts
                .into_iter()
                .filter(|part| !part.is_empty())
                .filter_map(|part| part.into_doc(state.clone()))
                .collect(),
        ))
    }
}

impl<'em> IntoDoc<'em> for ParPart<Content<'em>> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem<'em>> {
        match self {
            Self::Line(l) => l.into_doc(state),
            Self::Command(c) => c.into_doc(state),
        }
    }
}

impl<'em> IntoDoc<'em> for Vec<Content<'em>> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem<'em>> {
        Some(DocElem::Content(
            self.into_iter()
                .filter_map(|c| c.into_doc(state.clone()))
                .collect(),
        ))
    }
}

impl<'em> IntoDoc<'em> for Content<'em> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem<'em>> {
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
                name,
                plus: pluses != 0,
                attrs,
                args: {
                    inline_args
                        .into_iter()
                        .chain(remainder_arg.into_iter())
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
                word: Text::from(verbatim),
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

impl<'em> IntoDoc<'em> for Sugar<'em> {
    fn into_doc(self, state: DocStackState) -> Option<DocElem<'em>> {
        Some({
            let name = Text::from(self.call_name());
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
    use crate::parser;

    use super::*;

    fn assert_structure(name: &str, input: &str, expected: &str) {
        let src = textwrap::dedent(input);
        let doc: Doc = parser::parse(name, &src).unwrap().into();
        assert_eq!(expected, doc.repr(), "{name}");
    }

    #[test]
    fn into_doc() {
        assert_structure("empty", "", "[]");
        assert_structure("short", "foo", ".p{Word(foo)}");
        assert_structure(
            "long",
            r#"
                this is how to be a heart breaker
                _boys, they_ **like** `a` =little= ==danger==

                we'll get him !falling! for a stranger, a player
                singing---I lo-lo-love--you
                at~least~~I ~ think ~~ I ~do~~
            "#,
            "[.p{[Word(this)|Word(is)|Word(how)|Word(to)|Word(be)|Word(a)|Word(heart)|Word(breaker)|.it{[Word(boys,)|Word(they)]}|.bf{Word(like)}|.tt{Word(a)}|.sc{Word(little)}|.af{Word(danger)}]}|.p{[Word(we'll)|Word(get)|Word(him)|Word(falling)|Word(for)|Word(a)|Word(stranger,)|Word(a)|Word(player)|Word(singing)|---|Word(I)|Word(lo)|-|Word(lo)|-|Word(love)|--|Word(you)|Word(at)|~|Word(least)|~~|Word(I)|Word(think)|Word(I)|Word(do)]}]",
        );
    }

    #[test]
    fn into_doc_headings() {
        assert_structure(
            "headings",
            "
                # h1\n
                ## h2\n
                ### h3\n
                #### h4\n
                ##### h5\n
                ###### h6\n
                #+ h1\n
                ##+ h2\n
                ###+ h3\n
                ####+ h4\n
                #####+ h5\n
                ######+ h6\n
            ",
            "[.h1{Word(h1)}|.h2{Word(h2)}|.h3{Word(h3)}|.h4{Word(h4)}|.h5{Word(h5)}|.h6{Word(h6)}|.h1+{Word(h1)}|.h2+{Word(h2)}|.h3+{Word(h3)}|.h4+{Word(h4)}|.h5+{Word(h5)}|.h6+{Word(h6)}]",
        );
    }

    #[test]
    fn into_doc_commands() {
        assert_structure(
            "inline-single",
            ".get{ready here we come}",
            ".get{[Word(ready)|Word(here)|Word(we)|Word(come)]}",
        );
        assert_structure(
            "inline-multi",
            ".its{time}{to have}: some fun",
            ".its{Word(time)}{[Word(to)|Word(have)]}{[Word(some)|Word(fun)]}",
        );
        assert_structure(
            "remainder-single",
            ".you: know the world is ours",
            ".you{[Word(know)|Word(the)|Word(world)|Word(is)|Word(ours)]}",
        );
        assert_structure(
            "trailer-single",
            "
                .so:
                    now you're going down
            ",
            ".so{[Word(now)|Word(you're)|Word(going)|Word(down)]}",
        );
        assert_structure(
            "multi-trailer-single",
            "
                .so:
                    now you're going down
            ",
            ".so{[Word(now)|Word(you're)|Word(going)|Word(down)]}",
        );
        assert_structure(
            "trailer-multi",
            "
                .cos{that is}:
                    what
                ::
                    we say
            ",
            ".cos{[Word(that)|Word(is)]}{Word(what)}{[Word(we)|Word(say)]}",
        );
        assert_structure(
            "p-single-pars",
            "
                .p:
                    we are

                .p:
                    operation doomsday
            ",
            "[.p{[Word(we)|Word(are)]}|.p{[Word(operation)|Word(doomsday)]}]",
        );
        assert_structure(
            "p-multi-in-par",
            "
                .p:
                    we are
                .p:
                    operation doomsday
            ",
            ".p{[.p{[Word(we)|Word(are)]}|.p{[Word(operation)|Word(doomsday)]}]}",
        );
        assert_structure(
            "empty-preserved",
            "
                .we{}{are}{}{operation}{}{doomsday}{}
            ",
            ".we{[]}{Word(are)}{[]}{Word(operation)}{[]}{Word(doomsday)}{[]}",
        );
    }

    #[test]
    fn into_doc_comments() {
        assert_structure("line-comment", "// on this final night", "[]");
        assert_structure(
            "line-comment-in-arg",
            ".it: // on this final night",
            ".it{[]}",
        );
        assert_structure(
            "line-comment-inline",
            "before you // disappear",
            ".p{[Word(before)|Word(you)]}",
        );
        assert_structure("nested-comment", "/* forget all your worries */", "[]");
        assert_structure(
            "nested-comment-inline",
            "and let go of /* all */ your fears",
            ".p{[Word(and)|Word(let)|Word(go)|Word(of)|Word(your)|Word(fears)]}",
        );
    }
}
