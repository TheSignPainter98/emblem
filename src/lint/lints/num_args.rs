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
                        return vec![
                            Log::warn(format!(
                                "too {} arguments passed to .{name}",
                                if num_args > *max { "many" } else { "few" }
                            ))
                            .src(Src::new(loc).annotate(Note::info(
                                invocation_loc,
                                format!(
                                    "expected {max} {}",
                                    util::plural(*max, "argument", "arguments")
                                ),
                            ))),
                        ];
                    } else if num_args > *max {
                        return vec![
                            Log::warn(format!("too many arguments passed to .{name}")).src(
                                Src::new(loc).annotate(Note::info(
                                    invocation_loc,
                                    format!(
                                        "expected at most {} {}",
                                        max,
                                        util::plural(*max, "argument", "arguments")
                                    ),
                                )),
                            ),
                        ];
                    } else if num_args < *min {
                        return vec![
                            Log::warn(format!("too few arguments passed to .{name}")).src(
                                Src::new(loc).annotate(Note::info(
                                    invocation_loc,
                                    format!(
                                        "expected at least {} {}",
                                        min,
                                        util::plural(*min, "argument", "arguments")
                                    ),
                                )),
                            ),
                        ];
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
