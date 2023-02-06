use lazy_static::lazy_static;
use regex::Regex;

use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::log::{Log, Note, Src};

pub struct CommandNaming {}

impl CommandNaming {
    pub fn new() -> Self {
        Self {}
    }
}

lazy_static! {
    static ref CONFORMANT_NAME: Regex = Regex::new(r"^[a-z0-9-]*?$").unwrap();
}

impl<'i> Lint<'i> for CommandNaming {
    fn id(&self) -> &'static str {
        "command-naming"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Option<Log<'i>> {
        match content {
            Content::Command {
                name,
                loc,
                invocation_loc,
                ..
            } => {
                let name = name.as_ref();
                if !CONFORMANT_NAME.is_match(name) {
                    return Some(
                        Log::warn(format!(
                            "commands should be lowercase with dashes: got .{name}"
                        ))
                        .src(Src::new(loc).annotate(Note::help(
                            invocation_loc,
                            format!("try changing this to .{}", name.to_lowercase()),
                        )))
                        .note("command-names are case-insensitive but lowercase reads more fluidly"),
                    );
                }

                None
            }
            Content::Word { .. }
            | Content::Whitespace { .. }
            | Content::Dash { .. }
            | Content::Glue { .. }
            | Content::Verbatim { .. }
            | Content::Comment { .. }
            | Content::MultiLineComment { .. } => None,
        }
    }
}
