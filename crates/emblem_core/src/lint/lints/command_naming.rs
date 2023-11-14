use crate::ast::parsed::Content;
use crate::context::file_content::FileSlice;
use crate::lint::{Lint, LintId};
use crate::log::{Log, Note, Src};
use crate::Version;
use derive_new::new;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Clone, new)]
pub struct CommandNaming {}

lazy_static! {
    static ref CONFORMANT_NAME: Regex = Regex::new(r"^[a-z0-9-]*?$").unwrap();
}

impl Lint for CommandNaming {
    fn min_version(&self) -> Version {
        Version::V1_0
    }

    fn id(&self) -> LintId {
        "command-naming".into()
    }

    fn analyse(&mut self, content: &Content) -> Vec<Log> {
        match content {
            Content::Command {
                name,
                loc,
                invocation_loc,
                ..
            } => {
                if !CONFORMANT_NAME.is_match(name.to_str()) {
                    return vec![Log::warn(format!(
                        "commands should be lowercase with dashes: got ‘.{name}’"
                    ))
                    .with_src(Src::new(loc).with_annotation(Note::help(
                        invocation_loc,
                        format!(
                            "try changing this to ‘.{}’",
                            name.to_str().to_lowercase().replace('_', "-")
                        ),
                    )))
                    .with_note(
                        "command-names are case-insensitive but lowercase reads more fluidly",
                    )];
                }

                vec![]
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::lint::lints::test::LintTest;

    #[test]
    fn lint() {
        LintTest::new("empty", CommandNaming::new())
            .input("")
            .passes();
        LintTest::new("ok", CommandNaming::new())
            .input(".foo")
            .passes();
        LintTest::new("with-dashes", CommandNaming::new())
            .input(".foo-bar")
            .passes();
        LintTest::new("with-underscores", CommandNaming::new())
            .input(".foo_bar")
            .causes(
                1,
                &[
                    r"commands should be lowercase with dashes: got ‘.foo_bar’",
                    r":1:1-8: try changing this to ‘.foo-bar’",
                ],
            );
        LintTest::new("uppercase", CommandNaming::new())
            .input(".FOO")
            .causes(
                1,
                &[
                    r"commands should be lowercase with dashes: got ‘.FOO’",
                    r":1:1-4: try changing this to ‘.foo’",
                ],
            );
        LintTest::new("uppercase-unicode", CommandNaming::new())
            .input(".Φoo")
            .causes(
                1,
                &[
                    r"commands should be lowercase with dashes: got ‘.Φoo’",
                    r":1:1-4: try changing this to ‘.φoo’",
                ],
            );
    }
}
