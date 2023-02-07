use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use crate::util;
use derive_new::new;
use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(new)]
pub struct NumAttrs {}

lazy_static! {
    static ref AFFECTED_COMMANDS: HashMap<&'static str, (usize, usize)> = {
        vec![
            ("cite", (1, 1)),
            ("toc", (0, 0)),
            ("bf", (0, 0)),
            ("it", (0, 0)),
            ("sc", (0, 0)),
            ("af", (0, 0)),
            ("dt", (0, 0)),
            ("tt", (0, 0)),
            ("h1", (0, 0)),
            ("h2", (0, 0)),
            ("h3", (0, 0)),
            ("h4", (0, 0)),
            ("h5", (0, 0)),
            ("h6", (0, 0)),
            ("if", (0, 0)),
        ]
        .into_iter()
        .collect()
    };
}

impl<'i> Lint<'i> for NumAttrs {
    fn id(&self) -> &'static str {
        "num-attrs"
    }

    fn analyse(&mut self, content: &Content<'i>) -> Option<Log<'i>> {
        match content {
            Content::Command {
                name,
                loc,
                invocation_loc,
                attrs,
                ..
            } => {
                if let Some((min, max)) = AFFECTED_COMMANDS.get(name.as_ref()) {
                    let num_attrs = attrs.as_ref().map(|a| a.args().len()).unwrap_or_default();

                    let report_loc = if let Some(attrs) = attrs {
                        attrs.loc()
                    } else {
                        invocation_loc
                    };

                    if *max == *min && num_attrs != *max {
                        return Some(
                            Log::warn(format!(
                                "too {} attributes passed to .{name}",
                                if num_attrs > *max { "many" } else { "few" }
                            ))
                            .src(Src::new(&loc).annotate(Note::info(
                                        report_loc,
                                format!(
                                    "expected {max} {}",
                                    util::plural(*max, "attribute", "attributes")
                                ),
                            ))),
                        );
                    } else if num_attrs > *max {
                        return Some(
                            Log::warn(format!("too many attributes passed to .{name}")).src(
                                Src::new(&loc).annotate(Note::info(
                                        report_loc,
                                    format!(
                                        "expected at most {} {}",
                                        max,
                                        util::plural(*max, "attribute", "attributes")
                                    ),
                                )),
                            ),
                        );
                    } else if num_attrs < *min {
                        return Some(
                            Log::warn(format!("too few attributes passed to .{name}")).src(
                                Src::new(&loc).annotate(Note::info(
                                        report_loc,
                                    format!(
                                        "expected at least {} {}",
                                        min,
                                        util::plural(*min, "attribute", "attributes")
                                    ),
                                )),
                            ),
                        );
                    }
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
