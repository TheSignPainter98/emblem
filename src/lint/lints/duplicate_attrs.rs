use std::collections::HashMap;

use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use derive_new::new;

#[derive(new)]
pub struct DuplicateAttrs {}

impl<'i> Lint<'i> for DuplicateAttrs {
    fn id(&self) -> &'static str {
        "duplicate-attrs"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Option<Log<'i>> {
        match content {
            Content::Command {
                loc,
                attrs: Some(attrs),
                ..
            } => {
                let mut first_seen: HashMap<&str, &crate::ast::parsed::Attr>  = HashMap::new();
                let mut dups = Vec::new();
                for attr in attrs.args() {
                    let name = attr.name();
                    if let Some(first) = first_seen.get(name) {
                        dups.push((attr, first.clone()));
                    } else {
                        first_seen.insert(name, attr);
                    }
                }

                if dups.is_empty() {
                    return None;
                }

                Some(Log::warn("duplicate attributes").src({
                    let mut src = Src::new(loc);
                    for (dup, def) in dups {
                        let name = dup.name();
                        src = src.annotate(Note::warn(dup.loc(), format!("found duplicate '{}' here", name)));
                        src = src.annotate(Note::info(def.loc(), format!("'{}' first defined here", name)));
                    }
                    src
                })
                    .help("remove multiple occurrences of the same attribute")
                    )
            }
            Content::Command { .. }
            | Content::Word { .. }
            | Content::Whitespace { .. }
            | Content::Dash { .. }
            | Content::Glue { .. }
            | Content::Verbatim { .. }
            | Content::Comment { .. }
            | Content::MultiLineComment { .. } => None,
        }
    }
}
