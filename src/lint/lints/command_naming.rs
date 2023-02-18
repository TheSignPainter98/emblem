use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use derive_new::new;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(new)]
pub struct CommandNaming {}

lazy_static! {
    static ref CONFORMANT_NAME: Regex = Regex::new(r"^[a-z0-9-]*?$").unwrap();
}

impl<'i> Lint<'i> for CommandNaming {
    fn id(&self) -> &'static str {
        "command-naming"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Vec<Log<'i>> {
        match content {
            Content::Command {
                name,
                loc,
                invocation_loc,
                ..
            } => {
                let name = name.as_str();
                if !CONFORMANT_NAME.is_match(name) {
                    return vec![Log::warn(format!(
                        "commands should be lowercase with dashes: got ‘.{name}’"
                    ))
                    .src(Src::new(loc).annotate(Note::help(
                        invocation_loc,
                        format!(
                            "try changing this to ‘.{}’",
                            name.to_lowercase().replace('_', "-")
                        ),
                    )))
                    .note("command-names are case-insensitive but lowercase reads more fluidly")];
                }

                vec![]
            }
            Content::Word { .. }
            | Content::Sugar(_)
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
                lint: CommandNaming::new(),
                num_problems: 0,
                matches: vec![],
                src: "",
            },
            LintTest {
                lint: CommandNaming::new(),
                num_problems: 0,
                matches: vec![],
                src: ".foo-bar",
            },
            LintTest {
                lint: CommandNaming::new(),
                num_problems: 1,
                matches: vec![
                    r"commands should be lowercase with dashes: got ‘.foo_bar’",
                    r":1:1-8: try changing this to ‘.foo-bar’",
                ],
                src: ".foo_bar",
            },
            LintTest {
                lint: CommandNaming::new(),
                num_problems: 1,
                matches: vec![
                    r"commands should be lowercase with dashes: got ‘.FOO’",
                    r":1:1-4: try changing this to ‘.foo’",
                ],
                src: ".FOO",
            },
            LintTest {
                lint: CommandNaming::new(),
                num_problems: 1,
                matches: vec![
                    r"commands should be lowercase with dashes: got ‘.Φoo’",
                    r":1:1-4: try changing this to ‘.φoo’",
                ],
                src: ".Φoo",
            },
        ];

        for test in tests {
            test.run();
        }
    }
}
