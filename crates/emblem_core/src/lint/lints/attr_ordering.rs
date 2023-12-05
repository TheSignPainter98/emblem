use crate::ast::parsed::{Attr, Content};
use crate::lint::{Lint, LintId};
use crate::log::{Log, Note, Src};
use crate::Version;
use derive_new::new;

#[derive(Clone, new)]
pub struct AttrOrdering {}

impl Lint for AttrOrdering {
    fn min_version(&self) -> Version {
        Version::V1_0
    }

    fn id(&self) -> LintId {
        "attr-ordering".into()
    }

    fn analyse(&mut self, content: &Content) -> Vec<Log> {
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
                                Log::warning("unnamed attribute after named attributes")
                                    .with_src({
                                        Src::new(loc)
                                            .with_annotation(Note::warn(attr_loc, "found here"))
                                    })
                                    .with_help("place unnamed attributes before named ones"),
                            ),
                        _ => {}
                    }
                }
                ret
            }
            Content::Shebang { .. }
            | Content::Command { .. }
            | Content::Sugar(_)
            | Content::Word { .. }
            | Content::Whitespace { .. }
            | Content::Dash { .. }
            | Content::Glue { .. }
            | Content::SpiltGlue { .. }
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
        LintTest::new("empty", AttrOrdering::new())
            .input("")
            .passes();
        LintTest::new("no-attrs", AttrOrdering::new())
            .input(".foo")
            .passes();
        LintTest::new("single-unnamed", AttrOrdering::new())
            .input(".foo[bar]")
            .passes();
        LintTest::new("many-unnamed", AttrOrdering::new())
            .input(".foo[bar,baz]")
            .passes();
        LintTest::new("mixed", AttrOrdering::new())
            .input(".foo[bar,baz=baz]")
            .passes();
        LintTest::new("many-named", AttrOrdering::new())
            .input(".foo[bar=bar,baz=baz]")
            .passes();
        LintTest::new("named-before-unnamed", AttrOrdering::new())
            .input(".foo[bar=bar,baz]")
            .causes(
                1,
                &[
                    "unnamed attribute after named attribute",
                    ":1:14-16: found here",
                ],
            );
        LintTest::new("named-before-many-unnamed", AttrOrdering::new())
            .input(".foo[bar=bar,baz,quux]")
            .causes(
                2,
                &[
                    "unnamed attribute after named attribute",
                    ":1:(14-16|18-21): found here",
                ],
            );
        LintTest::new("many-incorrectly-mixed", AttrOrdering::new())
            .input(".foo[bar=bar,baz,quux=quux,corge]")
            .causes(
                2,
                &[
                    "unnamed attribute after named attribute",
                    ":1:(14-16|28-32): found here",
                ],
            );
    }
}
