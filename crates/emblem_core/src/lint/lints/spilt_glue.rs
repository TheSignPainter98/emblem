use crate::ast::parsed::Content;
use crate::lint::{Lint, LintId};
use crate::log::{Log, Note, Src};
use derive_new::new;

#[derive(new)]
pub struct SpiltGlue {}

impl Lint for SpiltGlue {
    fn id(&self) -> LintId {
        "spilt-glue".into()
    }

    fn analyse(&mut self, content: &Content) -> Vec<Log> {
        match content {
            Content::SpiltGlue { loc, .. } => {
                vec![Log::warn("glue does not connect text fragments")
                    .with_src(Src::new(loc).with_annotation(Note::info(loc, "found here")))]
            }
            Content::Shebang { .. }
            | Content::Command { .. }
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
                lint: SpiltGlue::new(),
                num_problems: 0,
                matches: vec![],
                src: "",
            },
            LintTest {
                lint: SpiltGlue::new(),
                num_problems: 0,
                matches: vec![],
                src: "a~b",
            },
            LintTest {
                lint: SpiltGlue::new(),
                num_problems: 1,
                matches: vec!["glue does not connect text fragments", ":1:2-3: found here"],
                src: "a ~b",
            },
            LintTest {
                lint: SpiltGlue::new(),
                num_problems: 1,
                matches: vec!["glue does not connect text fragments", ":1:2-3: found here"],
                src: "a~ b",
            },
            LintTest {
                lint: SpiltGlue::new(),
                num_problems: 1,
                matches: vec!["glue does not connect text fragments", ":1:2-4: found here"],
                src: "a ~ b",
            },
            LintTest {
                lint: SpiltGlue::new(),
                num_problems: 1,
                matches: vec!["glue does not connect text fragments", ":1:2-2: found here"],
                src: "a~",
            },
            LintTest {
                lint: SpiltGlue::new(),
                num_problems: 1,
                matches: vec!["glue does not connect text fragments", ":1:1-1: found here"],
                src: "~b",
            },
        ];

        for test in tests {
            test.run();
        }
    }
}
