use crate::Version;
use std::collections::HashMap;

use crate::ast::parsed::Content;
use crate::context::file_content::FileSlice;
use crate::lint::{Lint, LintId};
use crate::log::{Log, Note, Src};
use crate::parser::Location;
use derive_new::new;
use lazy_static::lazy_static;

#[derive(Clone, new)]
pub struct SugarUsage {}

#[derive(Clone)]
pub enum SugarType {
    Prefix(&'static str, Option<&'static str>),
    Delimiters(&'static str),
    // Surround {
    //     left: &'static str,
    //     right: &'static str,
    // },
}

impl SugarType {
    fn suggest(&self, name: &str, pluses: usize, loc: &Location, invocation_loc: &Location) -> Log {
        Log::warning(format!("syntactic sugar exists for .{name}"))
            .with_src(Src::new(loc).with_annotation(Note::help(invocation_loc, "found here")))
            .with_help(match self {
                Self::Prefix(_, Some(pre)) if pluses > 0 => format!("try using ‘{pre}’ instead"),
                Self::Prefix(pre, _) => format!("try using ‘{pre}’ instead"),
                Self::Delimiters(delim) => format!("try surrounding argument in ‘{delim}’ instead"),
                // Self::Surround { left, right } => {
                //     format!("try surrounding argument in ‘{left}’ and ‘{right}’ instead")
                // }
            })
    }
}

lazy_static! {
    static ref CALLS_TO_SUGARS: HashMap<&'static str, SugarType> = [
        ("it", SugarType::Delimiters("_")),
        ("bf", SugarType::Delimiters("**")),
        ("tt", SugarType::Delimiters("`")),
        ("sc", SugarType::Delimiters("=")),
        ("af", SugarType::Delimiters("==")),
        ("h1", SugarType::Prefix("#", Some("#+"))),
        ("h2", SugarType::Prefix("##", Some("##+"))),
        ("h3", SugarType::Prefix("###", Some("###+"))),
        ("h4", SugarType::Prefix("####", Some("####+"))),
        ("h5", SugarType::Prefix("#####", Some("#####+"))),
        ("h6", SugarType::Prefix("######", Some("######+"))),
        ("mark", SugarType::Prefix("@", None)),
        ("ref", SugarType::Prefix("#", None)),
    ]
    .into();
}

impl Lint for SugarUsage {
    fn min_version(&self) -> Version {
        Version::V1_0
    }

    fn id(&self) -> LintId {
        "sugar-usage".into()
    }

    fn analyse(&mut self, content: &Content) -> Vec<Log> {
        match content {
            Content::Command {
                name,
                inline_args,
                remainder_arg,
                trailer_args,
                loc,
                invocation_loc,
                pluses,
                attrs,
                ..
            } => {
                if let Some(expected) = CALLS_TO_SUGARS.get(name.to_str()) {
                    let attrs = attrs.iter().map(|a| a.args()).next().unwrap_or_default();
                    match (attrs, &inline_args[..], &remainder_arg, &trailer_args[..]) {
                        // A single argument or attr is suspicious
                        ([_], [], None, []) | ([], [_], None, []) | ([], [], Some(_), []) => {
                            return vec![expected.suggest(
                                name.to_str(),
                                *pluses,
                                loc,
                                invocation_loc,
                            )];
                        }
                        ([], [], None, [a]) => {
                            let [p] = &a[..] else {
                                return vec![];
                            };
                            if let [_] = &p.parts[..] {
                                return vec![expected.suggest(
                                    name.to_str(),
                                    *pluses,
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
            Content::Shebang { .. }
            | Content::Sugar(_)
            | Content::Word { .. }
            | Content::Whitespace { .. }
            | Content::Dash { .. }
            | Content::Glue { .. }
            | Content::SpiltGlue { .. }
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
        LintTest::new("empty", SugarUsage::new()).input("").passes();
        LintTest::new("word", SugarUsage::new())
            .input("foo")
            .passes();
        LintTest::new("command", SugarUsage::new())
            .input(".foo")
            .passes();
        for (call, sugar) in CALLS_TO_SUGARS.iter() {
            LintTest::new("missing-sugar", SugarUsage::new())
                .input(format!(".{call}"))
                .passes();
            LintTest::new("sugarable-with-empty-attrs", SugarUsage::new())
                .input(format!(".{call}[]"))
                .passes();
            LintTest::new("sugarable-with-many-attrs", SugarUsage::new())
                .input(format!(".{call}[foo,bar]"))
                .passes();
            LintTest::new("sugarable-with-many-args", SugarUsage::new())
                .input(format!(".{call}{{foo}}{{bar}}"))
                .passes();
            match sugar {
                SugarType::Prefix(pre, alternative_pre) => {
                    LintTest::new("sugarable-call", SugarUsage::new())
                        .input(format!(".{call}{{foo}}"))
                        .causes(
                            1,
                            &[
                                &format!(r"syntactic sugar exists for \.{call}"),
                                &format!(":1:1-{}: found here", 1 + call.len()),
                                &format!("try using ‘{}’ instead", pre.replace('*', r"\*")),
                            ],
                        );
                    if let Some(alternative_pre) = alternative_pre {
                        LintTest::new("sugarable-with-plus", SugarUsage::new())
                            .input(format!(".{call}+{{foo}}"))
                            .causes(
                                1,
                                &[
                                    &format!(r"syntactic sugar exists for \.{call}+"),
                                    &format!(":1:1-{}: found here", 2 + call.len()),
                                    &format!(
                                        "try using ‘{}’ instead",
                                        alternative_pre.replace('*', r"\*").replace('+', r"\+")
                                    ),
                                ],
                            );
                    }
                    LintTest::new("sugarable-with-attr", SugarUsage::new())
                        .input(format!(".{call}[foo]"))
                        .causes(
                            1,
                            &[
                                &format!(r"syntactic sugar exists for \.{call}+"),
                                &format!(":1:1-{}: found here", 1 + call.len()),
                                &format!(
                                    "try using ‘{}’ instead",
                                    pre.replace('*', r"\*").replace('+', r"\+")
                                ),
                            ],
                        );
                }
                SugarType::Delimiters(delim) => {
                    LintTest::new("sugarable-with-arg", SugarUsage::new())
                        .input(format!(".{call}{{foo}}"))
                        .causes(
                            1,
                            &[
                                &format!(r"syntactic sugar exists for \.{call}"),
                                &format!(":1:1-{}: found here", 1 + call.len()),
                                &format!(
                                    "try surrounding argument in ‘{}’ instead",
                                    delim.replace('*', r"\*")
                                ),
                            ],
                        );
                } // SugarType::Surround { left, right } => {
                  //     LintTest::new("sugarable-with-arg", SugarUsage::new())
                  //         .input(format!(".{call}{{foo}}"))
                  //         .causes(1, &["x"]);
                  // }
            }
        }
    }
}
