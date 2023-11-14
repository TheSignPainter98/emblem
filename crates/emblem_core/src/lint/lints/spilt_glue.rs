use crate::ast::parsed::Content;
use crate::lint::{Lint, LintId};
use crate::log::{Log, Note, Src};
use crate::Version;
use derive_new::new;

#[derive(Clone, new)]
pub struct SpiltGlue {}

impl Lint for SpiltGlue {
    fn min_version(&self) -> Version {
        Version::V1_0
    }

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
        LintTest::new("empty", SpiltGlue::new()).input("").passes();
        LintTest::new("valid-glue", SpiltGlue::new())
            .input("a~b")
            .passes();
        LintTest::new("spilt-left", SpiltGlue::new())
            .input("a ~b")
            .causes(
                1,
                &["glue does not connect text fragments", ":1:2-3: found here"],
            );
        LintTest::new("spilt-right", SpiltGlue::new())
            .input("a~ b")
            .causes(
                1,
                &["glue does not connect text fragments", ":1:2-3: found here"],
            );
        LintTest::new("double-spilt", SpiltGlue::new())
            .input("a ~ b")
            .causes(
                1,
                &["glue does not connect text fragments", ":1:2-4: found here"],
            );
        LintTest::new("unbonded-right", SpiltGlue::new())
            .input("a~")
            .causes(
                1,
                &["glue does not connect text fragments", ":1:2-2: found here"],
            );
        LintTest::new("unbonded-left", SpiltGlue::new())
            .input("~b")
            .causes(
                1,
                &["glue does not connect text fragments", ":1:1-1: found here"],
            );
    }
}
