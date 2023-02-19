use crate::ast::parsed::{Attr, Content};
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use derive_new::new;

#[derive(new)]
pub struct AttrOrdering {}

impl<'i> Lint<'i> for AttrOrdering {
    fn id(&self) -> &'static str {
        "attr-ordering"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Vec<Log<'i>> {
        match content {
            Content::Command {
                loc,
                attrs: Some(attrs),
                ..
            } => {
                enum ExpectationState {
                    Unnamed,
                    Named,
                }

                let mut ret = Vec::new();
                let mut state = ExpectationState::Unnamed;
                for attr in attrs.args() {
                    match (&state, &attr) {
                        (&ExpectationState::Unnamed, &Attr::Named { .. }) => {
                            state = ExpectationState::Named;
                        }
                        (&ExpectationState::Named, &Attr::Unnamed { loc: attr_loc, .. }) => ret
                            .push(
                                Log::warn("unnamed attribute after named attributes")
                                    .src({
                                        Src::new(loc).annotate(Note::warn(attr_loc, "found here"))
                                    })
                                    .help("place unnamed attributes before named ones"),
                            ),
                        _ => {}
                    }
                }
                ret
            }
            Content::Command { .. }
            | Content::Sugar(_)
            | Content::Word { .. }
            | Content::Whitespace { .. }
            | Content::Dash { .. }
            | Content::Glue { .. }
            | Content::Verbatim { .. }
            | Content::Comment { .. }
            | Content::MultiLineComment { .. } => vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lint::lints::test::LintTest;

    #[test]
    fn lint() {
        let tests = [
            LintTest {
                lint: AttrOrdering::new(),
                num_problems: 0,
                matches: vec![],
                src: "",
            },
            LintTest {
                lint: AttrOrdering::new(),
                num_problems: 0,
                matches: vec![],
                src: ".foo",
            },
            LintTest {
                lint: AttrOrdering::new(),
                num_problems: 0,
                matches: vec![],
                src: ".foo[bar]",
            },
            LintTest {
                lint: AttrOrdering::new(),
                num_problems: 0,
                matches: vec![],
                src: ".foo[bar,baz]",
            },
            LintTest {
                lint: AttrOrdering::new(),
                num_problems: 0,
                matches: vec![],
                src: ".foo[bar,baz=baz]",
            },
            LintTest {
                lint: AttrOrdering::new(),
                num_problems: 0,
                matches: vec![],
                src: ".foo[bar=bar,baz=baz]",
            },
            LintTest {
                lint: AttrOrdering::new(),
                num_problems: 1,
                matches: vec![
                    "unnamed attribute after named attribute",
                    ":1:14-16: found here",
                ],
                src: ".foo[bar=bar,baz]",
            },
            LintTest {
                lint: AttrOrdering::new(),
                num_problems: 2,
                matches: vec![
                    "unnamed attribute after named attribute",
                    ":1:(14-16|18-21): found here",
                ],
                src: ".foo[bar=bar,baz,quux]",
            },
            LintTest {
                lint: AttrOrdering::new(),
                num_problems: 2,
                matches: vec![
                    "unnamed attribute after named attribute",
                    ":1:(14-16|28-32): found here",
                ],
                src: ".foo[bar=bar,baz,quux=quux,corge]",
            },
        ];

        for test in tests {
            test.run();
        }
    }
}
