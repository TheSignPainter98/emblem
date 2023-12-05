use crate::ast::parsed::Content;
use crate::context::file_content::FileSlice;
use crate::lint::{Lint, LintId};
use crate::log::{Log, Note, Src};
use crate::util;
use crate::Version;
use derive_new::new;
use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Clone, new)]
pub struct NumAttrs {}

lazy_static! {
    static ref AFFECTED_COMMANDS: HashMap<&'static str, (usize, usize)> = {
        vec![
            ("cite", (1, 1)),
            ("mark", (1, 1)),
            ("ref", (1, 1)),
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

impl Lint for NumAttrs {
    fn min_version(&self) -> Version {
        Version::V1_0
    }

    fn id(&self) -> LintId {
        "num-attrs".into()
    }

    fn analyse(&mut self, content: &Content) -> Vec<Log> {
        match content {
            Content::Command {
                name,
                loc,
                invocation_loc,
                attrs,
                ..
            } => {
                if let Some((min, max)) = AFFECTED_COMMANDS.get(name.to_str()) {
                    let num_attrs = attrs.as_ref().map(|a| a.args().len()).unwrap_or_default();

                    let report_loc = if let Some(attrs) = attrs {
                        attrs.loc()
                    } else {
                        invocation_loc
                    };

                    if *max == *min && num_attrs != *max {
                        return vec![Log::warning(format!(
                            "too {} attributes passed to .{name}",
                            if num_attrs > *max { "many" } else { "few" }
                        ))
                        .with_src(Src::new(loc).with_annotation(Note::info(
                            report_loc,
                            if *max == 0 {
                                format!(
                                    "expected no {}",
                                    util::plural(*max, "attribute", "attributes")
                                )
                            } else {
                                format!(
                                    "expected {max} {}",
                                    util::plural(*max, "attribute", "attributes")
                                )
                            },
                        )))];
                    } else if num_attrs > *max {
                        return vec![Log::warning(format!(
                            "too many attributes passed to .{name}"
                        ))
                        .with_src(Src::new(loc).with_annotation(Note::info(
                            report_loc,
                            format!(
                                "expected at most {} {}",
                                max,
                                util::plural(*max, "attribute", "attributes")
                            ),
                        )))];
                    } else if num_attrs < *min {
                        return vec![
                            Log::warning(format!("too few attributes passed to .{name}")).with_src(
                                Src::new(loc).with_annotation(Note::info(
                                    report_loc,
                                    format!(
                                        "expected at least {} {}",
                                        min,
                                        util::plural(*min, "attribute", "attributes")
                                    ),
                                )),
                            ),
                        ];
                    }
                }

                vec![]
            }
            Content::Shebang { .. }
            | Content::Word { .. }
            | Content::Sugar(_)
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

    fn test_command(name: &str, num_pluses: usize, num_attrs: Option<(usize, usize)>) -> String {
        let mut args = vec![".", name];
        args.resize(args.len() + num_pluses, "+");
        if let Some((num_ordered_attrs, num_unordered_attrs)) = num_attrs {
            args.push("[");
            for i in 0..num_ordered_attrs {
                if i > 0 {
                    args.push(",");
                }
                args.push("foo");
            }

            for i in 0..num_unordered_attrs {
                if num_ordered_attrs > 0 || i > 0 {
                    args.push(",");
                }
                args.push("foo=bar");
            }

            args.push("]");
        }

        args.concat()
    }

    #[test]
    fn lint() {
        for (command, (min, max)) in AFFECTED_COMMANDS.iter() {
            let valid = *min..=*max;
            let start = if *min > 0 { min - 1 } else { *min };
            let end = max + 1;

            for pluses in 0..=2 {
                LintTest::new("bare", NumAttrs::new())
                    .input(test_command(command, pluses, None))
                    .causes(!valid.contains(&0) as u32, &["x"]);

                for num_ordered in start..=end {
                    for num_unordered in start..=end {
                        let tot = num_ordered + num_unordered;

                        LintTest::new("with-attrs", NumAttrs::new())
                            .input(test_command(
                                command,
                                pluses,
                                Some((num_ordered, num_unordered)),
                            ))
                            .causes(
                                !valid.contains(&(num_ordered + num_unordered)) as u32,
                                &[
                                    &if tot < *min {
                                        format!(r"too few attributes passed to \.{}", command)
                                    } else {
                                        format!(r"too many attributes passed to \.{}", command)
                                    },
                                    &{
                                        let start_col = 2 + command.len() + pluses;
                                        let end_col = start_col
                                            + 4 * num_ordered
                                            + 8 * num_unordered
                                            + (tot == 0) as usize;

                                        if *max == 0 {
                                            format!(
                                                r":1:{}-{}: expected no attributes",
                                                start_col, end_col,
                                            )
                                        } else if *max == *min {
                                            format!(
                                                r":1:{}-{}: expected {} {}",
                                                start_col,
                                                end_col,
                                                *min,
                                                util::plural(*min, "attribute", "attributes"),
                                            )
                                        } else if tot < *min {
                                            format!(
                                                r":1:{}-{}: expected at least {} {}",
                                                start_col,
                                                end_col,
                                                *min,
                                                util::plural(*min, "attribute", "attributes"),
                                            )
                                        } else {
                                            format!(
                                                r":1:{}-{}: expected at most {} {}",
                                                start_col,
                                                end_col,
                                                *max,
                                                util::plural(*min, "attribute", "attributes"),
                                            )
                                        }
                                    },
                                ],
                            );
                    }
                }
            }
        }
    }

    #[test]
    fn no_problems_by_default() {
        LintTest::new("default", NumAttrs::new()).input("").passes()
    }

    #[test]
    fn unaffected_ignored() {
        for pluses in 0..=2 {
            LintTest::new("bare", NumAttrs::new())
                .input(test_command("foo", pluses, None))
                .passes();

            for num_ordered in 0..=2 {
                for num_unordered in 0..=3 {
                    LintTest::new("with-attrs", NumAttrs::new())
                        .input(test_command(
                            "foo",
                            pluses,
                            Some((num_ordered, num_unordered)),
                        ))
                        .passes();
                }
            }
        }
    }
}
