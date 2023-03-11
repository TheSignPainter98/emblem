use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use crate::util;
use derive_new::new;

#[derive(new)]
pub struct NumArgs {}

lazy_static! {
    static ref AFFECTED_COMMANDS: HashMap<&'static str, (usize, usize)> = {
        vec![
            ("toc", (0, 0)),
            ("bf", (1, 1)),
            ("it", (1, 1)),
            ("sc", (1, 1)),
            ("af", (1, 1)),
            ("dt", (1, 1)),
            ("tt", (1, 1)),
            ("h1", (1, 1)),
            ("h2", (1, 1)),
            ("h3", (1, 1)),
            ("h4", (1, 1)),
            ("h5", (1, 1)),
            ("h6", (1, 1)),
            ("if", (2, 3)),
        ]
        .into_iter()
        .collect()
    };
}

impl<'i> Lint<'i> for NumArgs {
    fn id(&self) -> &'static str {
        "num-args"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Vec<Log<'i>> {
        match content {
            Content::Command {
                name,
                inline_args,
                remainder_arg,
                trailer_args,
                loc,
                invocation_loc,
                ..
            } => {
                if let Some((min, max)) = AFFECTED_COMMANDS.get(name.as_str()) {
                    let num_args =
                        inline_args.len() + remainder_arg.iter().len() + trailer_args.len();

                    if *max == *min && num_args != *max {
                        return vec![Log::warn(format!(
                            "too {} arguments passed to .{name}",
                            if num_args > *max { "many" } else { "few" }
                        ))
                        .with_src(Src::new(loc).with_annotation(Note::info(
                            invocation_loc,
                            if *max == 0 {
                                format!(
                                    "expected no {}",
                                    util::plural(*max, "argument", "arguments")
                                )
                            } else {
                                format!(
                                    "expected {max} {}",
                                    util::plural(*max, "argument", "arguments")
                                )
                            },
                        )))];
                    } else if num_args > *max {
                        return vec![Log::warn(format!("too many arguments passed to .{name}"))
                            .with_src(Src::new(loc).with_annotation(Note::info(
                                invocation_loc,
                                format!(
                                    "expected at most {} {}",
                                    max,
                                    util::plural(*max, "argument", "arguments")
                                ),
                            )))];
                    } else if num_args < *min {
                        return vec![Log::warn(format!("too few arguments passed to .{name}"))
                            .with_src(Src::new(loc).with_annotation(Note::info(
                                invocation_loc,
                                format!(
                                    "expected at least {} {}",
                                    min,
                                    util::plural(*min, "argument", "arguments")
                                ),
                            )))];
                    }
                }

                vec![]
            }
            Content::Word { .. }
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

    #[derive(Debug)]
    enum ArgsType {
        Inline { with_remainder: bool },
        Trailer,
    }

    fn test_command(name: &str, num_pluses: usize, num_args: usize, arg_type: &ArgsType) -> String {
        let mut args = vec![".", name];
        args.resize(args.len() + num_pluses, "+");
        match arg_type {
            ArgsType::Inline { with_remainder } => {
                let num_inline = match (*with_remainder, num_args) {
                    (_, 0) => 0,
                    (true, n) => n - 1,
                    (_, n) => n,
                };
                args.resize(args.len() + num_inline, "{foo}");
                if *with_remainder && num_args > 0 {
                    args.push(":foo");
                }
            }
            ArgsType::Trailer => {
                if num_args > 0 {
                    args.push(":\n\tfoo");
                }
                if num_args > 1 {
                    args.resize(args.len() + num_args - 1, "\n::\n\tfoo");
                }
            }
        }

        args.concat()
    }

    #[test]
    fn lint() {
        for (command, (min, max)) in AFFECTED_COMMANDS.iter() {
            let valid = *min..=*max;
            let min_args_to_test = if *min > 0 { min - 1 } else { *min };
            let max_args_to_test = max + 1;

            for arg_type in [
                ArgsType::Inline {
                    with_remainder: false,
                },
                ArgsType::Inline {
                    with_remainder: true,
                },
                ArgsType::Trailer,
            ] {
                for pluses in 0..=2 {
                    for num_args in min_args_to_test..=max_args_to_test {
                        LintTest {
                            lint: NumArgs::new(),
                            num_problems: !valid.contains(&num_args) as usize,
                            matches: vec![
                                &if num_args < *min {
                                    format!(r"too few arguments passed to \.{}", command)
                                } else {
                                    format!(r"too many arguments passed to \.{}", command)
                                },
                                &if *max == 0 {
                                    format!(
                                        r":1:1-{}: expected no arguments",
                                        1 + command.len() + pluses,
                                    )
                                } else if *max == *min {
                                    format!(
                                        r":1:1-{}: expected {} {}",
                                        1 + command.len() + pluses,
                                        *min,
                                        util::plural(*min, "argument", "arguments")
                                    )
                                } else if num_args < *min {
                                    format!(
                                        r":1:1-{}: expected at least {} {}",
                                        1 + command.len() + pluses,
                                        *min,
                                        util::plural(*min, "argument", "arguments")
                                    )
                                } else {
                                    format!(
                                        r":1:1-{}: expected at most {} {}",
                                        1 + command.len() + pluses,
                                        *max,
                                        util::plural(*max, "argument", "arguments")
                                    )
                                },
                            ],
                            src: &test_command(command, pluses, num_args, &arg_type),
                        }
                        .run();
                    }
                }
            }
        }
    }

    #[test]
    fn no_problems_by_default() {
        LintTest {
            lint: NumArgs::new(),
            num_problems: 0,
            matches: vec![],
            src: "",
        }
        .run();
    }

    #[test]
    fn unaffected_ignored() {
        for arg_type in [
            ArgsType::Inline {
                with_remainder: false,
            },
            ArgsType::Inline {
                with_remainder: true,
            },
            ArgsType::Trailer,
        ] {
            for pluses in 0..=2 {
                for num_args in 0..=3 {
                    LintTest {
                        lint: NumArgs::new(),
                        num_problems: 0,
                        matches: vec![],
                        src: &test_command("foo", pluses, num_args, &arg_type),
                    }
                    .run();
                }
            }
        }
    }
}
