use crate::ast::parsed::Content;
use crate::lint::{Lint, LintId};
use crate::log::{Log, Note, Src};
use derive_new::new;

#[derive(new)]
pub struct EmptyAttrs {}

impl Lint for EmptyAttrs {
    fn id(&self) -> LintId {
        "empty-attrs".into()
    }

    fn analyse(&mut self, content: &Content) -> Vec<Log> {
        match content {
            Content::Command {
                loc,
                attrs: Some(attrs),
                ..
            } => {
                if !attrs.args().is_empty() {
                    return vec![];
                }

                vec![Log::warn("empty attributes")
                    .with_src(Src::new(loc).with_annotation(Note::info(attrs.loc(), "found here")))]
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
        let tests = [
            LintTest {
                lint: EmptyAttrs::new(),
                num_problems: 0,
                matches: vec![],
                src: "",
            },
            LintTest {
                lint: EmptyAttrs::new(),
                num_problems: 0,
                matches: vec![],
                src: ".foo",
            },
            LintTest {
                lint: EmptyAttrs::new(),
                num_problems: 1,
                matches: vec![":1:5-6: found here"],
                src: ".foo[]",
            },
        ];

        for test in tests {
            test.run();
        }
    }
}
