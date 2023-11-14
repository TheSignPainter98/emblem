use crate::ast::parsed::Content;
use crate::lint::{Lint, LintId};
use crate::log::{Log, Note, Src};
use crate::Version;
use derive_new::new;

#[derive(Clone, new)]
pub struct EmptyAttrs {}

impl Lint for EmptyAttrs {
    fn min_version(&self) -> Version {
        Version::V1_0
    }

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
        LintTest::new("empty", EmptyAttrs::new()).input("").passes();
        LintTest::new("no-attrs", EmptyAttrs::new())
            .input(".foo")
            .passes();
        LintTest::new("empty-attrs", EmptyAttrs::new())
            .input(".foo[]")
            .causes(1, &[":1:5-6: found here"]);
    }
}
