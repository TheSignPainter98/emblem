use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::log::{Log, Note, Src};

pub struct ExcessiveArgs {}

impl ExcessiveArgs {
    pub fn new() -> Self {
        Self {}
    }
}

lazy_static! {
    // TODO(kcza): lint too few args!
    // TODO(kcza): do the exact same for attributes
    static ref AFFECTED_COMMANDS: HashMap<&'static str, usize> = {
        vec![
            ("bf", 1),
            ("it", 1),
            ("sc", 1),
            ("af", 1),
            ("dt", 1),
            ("tt", 1),
            ("toc", 0),
            ("if", 3),
            ("cite", 1),
            ("anchor", 1),
            ("ref", 1),
            ("h1", 1),
            ("h2", 1),
            ("h3", 1),
            ("h4", 1),
            ("h5", 1),
            ("h6", 1),
        ]
        .into_iter()
        .collect()
    };
}

impl<'i> Lint<'i> for ExcessiveArgs {
    fn id(&self) -> &'static str {
        "too-many-args"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Option<Log<'i>> {
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
                if let Some(max) = AFFECTED_COMMANDS.get(name.as_ref()) {
                    let num_style_args =
                        inline_args.len() + remainder_arg.iter().len() + trailer_args.len();
                    if num_style_args > *max {
                        return Some(
                            Log::warn(format!("too many arguments passed to .{name}",)).src(
                                Src::new(loc).annotate(Note::info(
                                    invocation_loc,
                                    format!("expected at most {max} arguments"),
                                )),
                            ),
                        );
                    }
                }

                None
            }
            Content::Word{ .. }
            | Content::Whitespace{ .. }
            | Content::Dash{ .. }
            | Content::Glue{ .. }
            | Content::Verbatim{ .. }
            | Content::Comment{ .. }
            | Content::MultiLineComment{ .. } => None
        }
    }
}
