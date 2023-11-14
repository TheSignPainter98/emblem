use crate::ast::parsed::{Content, Sugar};
use crate::lint::{Lint, LintId};
use crate::log::{Log, Note, Src};
use crate::Version;
use derive_new::new;

#[derive(Clone, new)]
pub struct EmphDelimiters {}

impl Lint for EmphDelimiters {
    fn min_version(&self) -> Version {
        Version::V1_0
    }

    fn id(&self) -> LintId {
        "emph-delimiters".into()
    }

    fn analyse(&mut self, content: &Content) -> Vec<Log> {
        match content {
            Content::Sugar(Sugar::Italic { delimiter, loc, .. }) if delimiter == "*" => {
                vec![Log::warn("asterisks used to delimit italic text").with_src(
                    Src::new(loc).with_annotation(Note::help(loc, "use underscores instead")),
                )]
            }
            Content::Sugar(Sugar::Bold { delimiter, loc, .. }) if delimiter == "__" => {
                vec![Log::warn("underscores used to delimit bold text").with_src(
                    Src::new(loc).with_annotation(Note::help(loc, "use asterisks instead")),
                )]
            }
            Content::Shebang { .. }
            | Content::Word { .. }
            | Content::Sugar(_)
            | Content::Command { .. }
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
        LintTest::new("empty", EmphDelimiters::new())
            .input("")
            .passes();
        LintTest::new("no-delimiters", EmphDelimiters::new())
            .input("foo")
            .passes();
        LintTest::new("good-italic-delimiters", EmphDelimiters::new())
            .input("_foo_")
            .passes();
        LintTest::new("good-bold-delimiters", EmphDelimiters::new())
            .input("**foo**")
            .passes();
        LintTest::new("bad-italic-delimiters", EmphDelimiters::new())
            .input("*foo*")
            .causes(
                1,
                &[
                    "asterisks used to delimit italic text",
                    ":1:1-5: use underscores instead",
                ],
            );
        LintTest::new("bad-bold-delimiters", EmphDelimiters::new())
            .input("__foo__")
            .causes(
                1,
                &[
                    "underscores used to delimit bold text",
                    ":1:1-7: use asterisks instead",
                ],
            );
    }
}
