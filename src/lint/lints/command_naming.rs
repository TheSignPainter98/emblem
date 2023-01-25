use lazy_static::lazy_static;
use regex::Regex;

use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::lint::Problem;

pub struct CommandNaming {}

impl CommandNaming {
    pub fn new() -> Self {
        Self {}
    }
}

lazy_static! {
    static ref CONFORMANT_NAME: Regex = Regex::new(r"^[a-z0-9-]*?$").unwrap();
}

impl Lint for CommandNaming {
    fn id(&self) -> &'static str {
        "command-naming"
    }

    fn analyse<'i>(&mut self, content: &Content<'i>) -> Option<Problem> {
        match content {
            Content::Command { name, .. } => {
                let name = name.as_ref();
                if !CONFORMANT_NAME.is_match(name) {
                    return Some(self.problem(format!(
                        "commands should be lowercase with dashes: got .{}",
                        name
                    )))
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
