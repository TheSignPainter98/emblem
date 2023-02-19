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
    suggested_delim: &str,
    loc: &Location<'i>,
    invocation_loc: &Location<'i>,
) -> Log<'i> {
    Log::warn("explicit styling call")
        .src(Src::new(loc).annotate(Note::help(
            invocation_loc,
            "syntactic sugar exists for this command",
        )))
        .help(format!(
            "try surrounding argument in ‘{suggested_delim}’ instead"
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
                        ([_], None, []) => return vec![emph_warning(delim, loc, invocation_loc)],
                        ([], Some(_), []) => return vec![emph_warning(delim, loc, invocation_loc)],
                        ([], None, [a]) => {
                            let [p] = &a[..] else { return vec![]; };
                            if let [_] = &p.parts[..] {
                                return vec![emph_warning(delim, loc, invocation_loc)];
                            }
                        }
                        _ => {}
                    }
                }
                vec![]
            }
            Content::Sugar(_)
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
        for (call, delim) in CALLS_TO_SUGARS.iter() {
            LintTest {
                lint: SugarUsage::new(),
                num_problems: 0,
                matches: vec![],
                src: &format!(".{call}"),
            }
            .run();
            LintTest {
                lint: SugarUsage::new(),
                num_problems: 0,
                matches: vec![],
                src: &format!(".{call}{{foo}}{{bar}}"),
            }
            .run();
            LintTest {
                lint: SugarUsage::new(),
                num_problems: 1,
                matches: vec![
                    "explicit styling call",
                    &format!(
                        ":1:1-{}: syntactic sugar exists for this command",
                        1 + call.len(),
                    ),
                    &format!(
                        "try surrounding argument in ‘{}’ instead",
                        delim.replace('*', r"\*")
                    ),
                ],
                src: &format!(".{call}{{foo}}"),
            }
            .run();
        }
    }
}
