use crate::ast::parsed::{Content, Sugar};
use crate::lint::{Lint, LintId};
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use crate::Version;
use derive_new::new;

#[derive(Clone, new)]
pub struct NumPluses {}

impl Lint for NumPluses {
    fn min_version(&self) -> Version {
        Version::V1_0
    }

    fn id(&self) -> LintId {
        "num-pluses".into()
    }

    fn analyse(&mut self, content: &Content) -> Vec<Log> {
        match content {
            Content::Command {
                loc,
                pluses,
                invocation_loc,
                ..
            } => {
                if *pluses <= 1 {
                    return vec![];
                }

                vec![self.message(loc, invocation_loc)]
            }
            Content::Sugar(Sugar::Heading {
                pluses,
                loc,
                invocation_loc,
                ..
            }) => {
                if *pluses <= 1 {
                    return vec![];
                }

                vec![self.message(loc, invocation_loc)]
            }
            Content::Shebang { .. }
            | Content::Word { .. }
            | Content::Sugar(_)
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

impl NumPluses {
    fn message(&self, loc: &Location, invocation_loc: &Location) -> Log {
        Log::warning("extra plus modifiers ignored").with_src(
            Src::new(loc)
                .with_annotation(Note::help(invocation_loc, "remove all but one plus symbol")),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lint::lints::test::LintTest;

    #[test]
    fn empty() {
        LintTest::new("empty", NumPluses::new()).input("").passes();
    }

    #[test]
    fn command() {
        LintTest::new("no-plus", NumPluses::new())
            .input(".foo")
            .passes();
        LintTest::new("single-plus", NumPluses::new())
            .input(".foo+")
            .passes();
        LintTest::new("double-plus", NumPluses::new())
            .input(".foo++")
            .causes(
                1,
                &[
                    "extra plus modifiers ignored",
                    ":1:1-6: remove all but one plus symbol",
                ],
            );
    }

    #[test]
    fn heading_sugar() {
        for level in 1..=6 {
            LintTest::new("no-plus", NumPluses::new())
                .input(format!("{} foo", "#".repeat(level)))
                .passes();
            LintTest::new("single-plus", NumPluses::new())
                .input(format!("{}+ foo", "#".repeat(level)))
                .passes();
            LintTest::new("double-plus", NumPluses::new())
                .input(format!("{}++ foo", "#".repeat(level)))
                .causes(
                    1,
                    &[
                        "extra plus modifiers ignored",
                        &format!(":1:1-{}: remove all but one plus symbol", level + 2),
                    ],
                );
        }
    }
}
