use crate::ast::parsed::{Content, Sugar};
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use derive_new::new;

#[derive(new)]
pub struct EmphDelimiters {}

impl<'i> Lint<'i> for EmphDelimiters {
    fn id(&self) -> &'static str {
        "emph-delimiters"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Vec<Log<'i>> {
        match content {
            Content::Sugar(Sugar::Italic { delimiter, loc, .. }) if *delimiter == "*" => {
                vec![Log::warn("asterisks used to delimit italic text").with_src(
                    Src::new(loc).with_annotation(Note::help(loc, "use underscores instead")),
                )]
            }
            Content::Sugar(Sugar::Bold { delimiter, loc, .. }) if *delimiter == "__" => {
                vec![Log::warn("underscores used to delimit bold text").with_src(
                    Src::new(loc).with_annotation(Note::help(loc, "use asterisks instead")),
                )]
            }
            Content::Word { .. }
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
        LintTest {
            lint: EmphDelimiters::new(),
            num_problems: 0,
            matches: vec![],
            src: "",
        }
        .run();
        LintTest {
            lint: EmphDelimiters::new(),
            num_problems: 0,
            matches: vec![],
            src: "foo",
        }
        .run();
        LintTest {
            lint: EmphDelimiters::new(),
            num_problems: 0,
            matches: vec![],
            src: "_foo_",
        }
        .run();
        LintTest {
            lint: EmphDelimiters::new(),
            num_problems: 0,
            matches: vec![],
            src: "**foo**",
        }
        .run();
        LintTest {
            lint: EmphDelimiters::new(),
            num_problems: 1,
            matches: vec![
                "underscores used to delimit bold text",
                ":1:1-7: use asterisks instead",
            ],
            src: "__foo__",
        }
        .run();
        LintTest {
            lint: EmphDelimiters::new(),
            num_problems: 1,
            matches: vec![
                "asterisks used to delimit italic text",
                ":1:1-5: use underscores instead",
            ],
            src: "*foo*",
        }
        .run();
    }
}
