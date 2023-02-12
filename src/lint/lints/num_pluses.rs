use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use derive_new::new;

#[derive(new)]
pub struct NumPluses {}

impl<'i> Lint<'i> for NumPluses {
    fn id(&self) -> &'static str {
        "num-pluses"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Vec<Log<'i>> {
        match content {
            Content::Command {
                loc,
                pluses,
                invocation_loc,
                ..
            } => {
                if *pluses >= 2 {
                    return vec![Log::warn("extra plus modifiers ignored").src(
                        Src::new(loc)
                            .annotate(Note::help(invocation_loc, "remove all but one plus symbol")),
                    )];
                }

                vec![]
            }
            Content::Word { .. }
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
        LintTest {
            lint: NumPluses::new(),
            num_problems: 0,
            matches: vec![],
            src: "",
        }
        .run();
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
}
