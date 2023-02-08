use crate::ast::parsed::{Attr, Content};
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use derive_new::new;

#[derive(new)]
pub struct AttrOrdering {}

impl<'i> Lint<'i> for AttrOrdering {
    fn id(&self) -> &'static str {
        "attr-ordering"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Vec<Log<'i>> {
        match content {
            Content::Command {
                loc,
                attrs: Some(attrs),
                ..
            } => {
                enum ExpectationState {
                    Unnamed,
                    Named,
                }

                let mut ret = Vec::new();
                let mut state = ExpectationState::Unnamed;
                for attr in attrs.args() {
                    match (&state, &attr) {
                        (&ExpectationState::Unnamed, &Attr::Named { .. }) => {
                            state = ExpectationState::Named;
                        }
                        (&ExpectationState::Named, &Attr::Unnamed { loc: attr_loc, .. }) => ret
                            .push(
                                Log::warn("unnamed attribute after named attributes")
                                    .src({
                                        Src::new(loc).annotate(Note::warn(attr_loc, "found here"))
                                    })
                                    .help("place unnamed attributes before named ones"),
                            ),
                        _ => {}
                    }
                }
                ret
            }
            Content::Command { .. }
            | Content::Word { .. }
            | Content::Whitespace { .. }
            | Content::Dash { .. }
            | Content::Glue { .. }
            | Content::Verbatim { .. }
            | Content::Comment { .. }
            | Content::MultiLineComment { .. } => vec![],
        }
    }
}
