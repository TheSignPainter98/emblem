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
                if let Some((min, max)) = AFFECTED_COMMANDS.get(name.as_ref()) {
                    let num_args =
                        inline_args.len() + remainder_arg.iter().len() + trailer_args.len();

                    if *max == *min && num_args != *max {
                        return vec![Log::warn(format!(
                            "too {} arguments passed to .{name}",
                            if num_args > *max { "many" } else { "few" }
                        ))
                        .src(Src::new(loc).annotate(Note::info(
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
                            .src(Src::new(loc).annotate(Note::info(
                                invocation_loc,
                                format!(
                                    "expected at most {} {}",
                                    max,
                                    util::plural(*max, "argument", "arguments")
                                ),
                            )))];
                    } else if num_args < *min {
                        return vec![Log::warn(format!("too few arguments passed to .{name}"))
                            .src(Src::new(loc).annotate(Note::info(
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
    use typed_arena::Arena;

    #[derive(Debug)]
    enum ArgType {
        Inline { with_remainder: bool },
        Trailer,
    }

    fn test_command(name: &str, num_stars: usize, num_args: usize, arg_type: &ArgType) -> String {
        let mut args = vec![".", name];
        for _ in 0..num_stars {
            args.push("*");
        }
        match arg_type {
            ArgType::Inline { with_remainder } => {
                let num_inline = match (*with_remainder, num_args) {
                    (_, 0) => 0,
                    (true, n) => n - 1,
                    (_, n) => n,
                };
                for _ in 0..num_inline {
                    args.push("{foo}");
                }
                if *with_remainder && num_args > 0 {
                    args.push(":foo");
                }
            }
            ArgType::Trailer => {
                if num_args > 0 {
                    args.push(":\n\tfoo");
                }
                for _ in 1..num_args {
                    args.push("\n::\n\tfoo");
                }
            }
        }

        args.concat()
    }

    #[test]
    fn lint() {
        let arena = Arena::new();

        let mut tests = vec![];
        for (command, (min, max)) in AFFECTED_COMMANDS.iter() {
            let valid = *min..=*max;
            let start = if *min > 0 { min - 1 } else { *min };
            let end = max + 1;

            for arg_type in [
                ArgType::Inline {
                    with_remainder: false,
                },
                ArgType::Inline {
                    with_remainder: true,
                },
                ArgType::Trailer,
            ] {
                for stars in 0..=2 {
                    for i in start..=end {
                        tests.push(LintTest {
                            lint: NumArgs::new(),
                            num_problems: !valid.contains(&i) as usize,
                            matches: vec![
                                arena.alloc(if i < *min {
                                    format!(r"too few arguments passed to \.{}", command)
                                } else {
                                    format!(r"too many arguments passed to \.{}", command)
                                }),
                                arena.alloc(if *max == 0 {
                                    format!(
                                        r":1:1-{}: expected no arguments",
                                        1 + command.len() + stars,
                                    )
                                } else if *max == *min {
                                    format!(
                                        r":1:1-{}: expected {} {}",
                                        1 + command.len() + stars,
                                        *min,
                                        util::plural(*min, "argument", "arguments")
                                    )
                                } else if i < *min {
                                    format!(
                                        r":1:1-{}: expected at least {} {}",
                                        1 + command.len() + stars,
                                        *min,
                                        util::plural(*min, "argument", "arguments")
                                    )
                                } else {
                                    format!(
                                        r":1:1-{}: expected at most {} {}",
                                        1 + command.len() + stars,
                                        *max,
                                        util::plural(*max, "argument", "arguments")
                                    )
                                }),
                            ],
                            src: arena.alloc(test_command(command, stars, i, &arg_type)),
                        });
                    }
                }
            }
        }

        for test in tests {
            test.run();
        }
    }
}
