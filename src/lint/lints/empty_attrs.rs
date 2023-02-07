use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use derive_new::new;

#[derive(new)]
pub struct EmptyAttrs {}

impl<'i> Lint<'i> for EmptyAttrs {
    fn id(&self) -> &'static str {
        "empty-attrs"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Option<Log<'i>> {
        match content {
            Content::Command { loc, attrs, .. } => {
                if attrs.is_none() {
                    return None;
                }
                let attrs = attrs.as_ref().unwrap();

                if attrs.args().is_empty() {
                    return Some(
                        Log::warn("empty attributes")
                            .src(Src::new(loc).annotate(Note::info(attrs.loc(), "found here"))),
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
