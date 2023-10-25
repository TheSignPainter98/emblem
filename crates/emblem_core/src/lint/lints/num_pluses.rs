use crate::ast::parsed::{Content, Sugar};
use crate::lint::{Lint, LintId};
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;

#[derive(new)]
pub struct NumPluses {}

impl Lint for NumPluses {
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
            | Content::InlineVerbatim { .. }
            | Content::Comment { .. }
            | Content::MultiLineComment { .. } => vec![],
        }
    }
}

impl NumPluses {
    fn message(&self, loc: &Location, invocation_loc: &Location) -> Log {
        Log::warn("extra plus modifiers ignored").with_src(
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
        LintTest {
            lint: NumPluses::new(),
            num_problems: 0,
            matches: vec![],
            src: "",
        }
        .run();
    }

    #[test]
    fn command() {
        LintTest {
            lint: NumPluses::new(),
            num_problems: 0,
            matches: vec![],
            src: ".foo",
        }
        .run();
        LintTest {
            lint: NumPluses::new(),
            num_problems: 0,
            matches: vec![],
            src: ".foo+",
        }
        .run();
        LintTest {
            lint: NumPluses::new(),
            num_problems: 1,
            matches: vec![
                "extra plus modifiers ignored",
                ":1:1-6: remove all but one plus symbol",
            ],
            src: ".foo++",
        }
        .run();
    }

    #[test]
    fn heading_sugar() {
        for level in 1..=6 {
            LintTest {
                lint: NumPluses::new(),
                num_problems: 0,
                matches: vec![],
                src: &format!("{} foo", "#".repeat(level)),
            }
            .run();
            LintTest {
                lint: NumPluses::new(),
                num_problems: 0,
                matches: vec![
                    "extra plus modifiers ignored",
                    ":1:1-6: remove all but one plus symbol",
                ],
                src: &format!("{}+ foo", "#".repeat(level)),
            }
            .run();
            LintTest {
                lint: NumPluses::new(),
                num_problems: 1,
                matches: vec![
                    "extra plus modifiers ignored",
                    &format!(":1:1-{}: remove all but one plus symbol", level + 2),
                ],
                src: &format!("{}++ foo", "#".repeat(level)),
            }
            .run();
        }
    }
}
