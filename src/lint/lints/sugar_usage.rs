use std::collections::HashMap;

use crate::ast::parsed::Content;
use crate::lint::Lint;
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;
use lazy_static::lazy_static;

#[derive(new)]
pub struct SugarUsage {}

lazy_static! {
    static ref CALLS_TO_SUGARS: HashMap<&'static str, &'static str> = [
        ("it", "_"),
        ("bf", "**"),
        ("tt", "`"),
        ("sc", "="),
        ("af", "=="),
    ]
    .into();
}

fn emph_warning<'i>(
    name: &str,
    suggested_delim: &str,
    loc: &Location<'i>,
    invocation_loc: &Location<'i>,
) -> Log<'i> {
    Log::warn("emphasis command used on single argument")
        .src(Src::new(loc).annotate(Note::info(
            invocation_loc,
            format!("syntactic sugar exists for '.{}'", name),
        )))
        .help(format!(
            "syntactic sugar exists for this, try surrounding the arg in '{}' instead",
            suggested_delim
        ))
}

impl<'i> Lint<'i> for SugarUsage {
    fn id(&self) -> &'static str {
        "sugar-usage"
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
                if let Some(delim) = CALLS_TO_SUGARS.get(name.as_str()) {
                    match (&inline_args[..], &remainder_arg, &trailer_args[..]) {
                        ([_], None, []) => {
                            return vec![emph_warning(name.as_str(), delim, loc, invocation_loc)]
                        }
                        ([], Some(_), []) => {
                            return vec![emph_warning(name.as_str(), delim, loc, invocation_loc)]
                        }
                        ([], None, [a]) => {
                            let [p] = &a[..] else { return vec![]; };
                            if let [_] = &p.parts[..] {
                                return vec![emph_warning(
                                    name.as_str(),
                                    delim,
                                    loc,
                                    invocation_loc,
                                )];
                            }
                        }
                        _ => {}
                    }
                }
                vec![]
            }
            Content::Sugar(_) => {
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

    #[test]
    fn lint() {
        LintTest {
            lint: SugarUsage::new(),
            num_problems: 0,
            matches: vec![],
            src: "",
        }
        .run();
        LintTest {
            lint: SugarUsage::new(),
            num_problems: 0,
            matches: vec![],
            src: "foo",
        }
        .run();
        LintTest {
            lint: SugarUsage::new(),
            num_problems: 0,
            matches: vec![],
            src: ".foo",
        }
        .run();
    }
}
