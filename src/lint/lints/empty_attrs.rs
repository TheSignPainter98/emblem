use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use derive_new::new;

#[derive(new)]
pub struct EmptyAttrs {}

impl<'i> Lint<'i> for EmptyAttrs {
    fn id(&self) -> &'static str {
        "empty-attrs"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Vec<Log<'i>> {
        match content {
            Content::Command {
                loc,
                attrs: Some(attrs),
                ..
            } => {
                if attrs.args().is_empty() {
                    return vec![Log::warn("empty attributes")
                        .src(Src::new(loc).annotate(Note::info(attrs.loc(), "found here")))];
                }

                vec![]
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
