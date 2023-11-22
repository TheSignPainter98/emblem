use annotate_snippets::{
    display_list::{
        DisplayAnnotationType, DisplayLine, DisplayList, DisplayRawLine, DisplayTextFragment,
        DisplayTextStyle, FormatOptions,
    },
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use derive_builder::Builder;
use emblem_core::{
    context::file_content::FileSlice,
    log::{Logger, MessageType},
    Error as EmblemError, Log, Result as EmblemResult, Verbosity,
};
use typed_arena::Arena;

#[derive(Default, Builder)]
pub struct PrettyLogger {
    #[builder(setter(into))]
    verbosity: Verbosity,

    #[builder(default)]
    colourise: bool,

    #[builder(setter(strip_option), default)]
    max_errors: Option<i32>,

    #[builder(setter(skip))]
    tot_errors: i32,

    #[builder(setter(skip))]
    tot_warnings: i32,
}

impl PrettyLogger {
    pub fn builder() -> PrettyLoggerBuilder {
        PrettyLoggerBuilder::default()
    }
}

impl Logger for PrettyLogger {
    fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    fn print(&mut self, log: Log) -> EmblemResult<()> {
        let msg_type = log.msg_type();

        if !self.verbosity.permits_printing(msg_type) {
            return Ok(());
        }

        let expected_string;
        let footer = {
            let mut footer = vec![];

            if let Some(help) = log.help() {
                footer.push(Annotation {
                    id: None,
                    label: Some(help),
                    annotation_type: AnnotationType::Help,
                });
            }

            if let Some(note) = log.note() {
                footer.push(Annotation {
                    id: None,
                    label: Some(note),
                    annotation_type: AnnotationType::Note,
                });
            }

            if let Some(expected) = log.expected() {
                let len = expected.len();

                expected_string = if len == 1 {
                    format!("expected {}", expected[0])
                } else {
                    let mut pretty_expected = Vec::new();
                    for (i, e) in expected.iter().enumerate() {
                        if i > 0 {
                            pretty_expected.push(if i < len - 1 { ", " } else { " or " })
                        }
                        pretty_expected.push(e);
                    }

                    format!("expected one of {}", pretty_expected.concat())
                };

                footer.push(Annotation {
                    id: None,
                    label: Some(&expected_string),
                    annotation_type: AnnotationType::Note,
                })
            }

            footer
        };

        let contexts = Arena::new();
        let snippet = Snippet {
            title: Some(Annotation {
                id: log.id().defined(),
                label: Some(log.msg()),
                annotation_type: convert_message_type(msg_type),
            }),
            slices: log
                .srcs()
                .iter()
                .map(|s| {
                    let context = contexts.alloc(s.loc().context());
                    Slice {
                        source: context.raw(),
                        line_start: s.loc().lines().0,
                        origin: Some(s.loc().file_name().as_ref()),
                        fold: true,
                        annotations: s
                            .annotations()
                            .iter()
                            .map(|a| SourceAnnotation {
                                annotation_type: convert_message_type(a.msg_type()),
                                label: a.msg(),
                                range: a.loc().indices_in(context),
                            })
                            .collect(),
                    }
                })
                .collect(),
            footer,
            opt: FormatOptions {
                color: self.colourise,
                ..Default::default()
            },
        };

        if log.is_explainable() {
            if !log.id().is_defined() {
                panic!("internal error: explainable message has no id")
            }

            let info_instruction = &format!(
                "For more information about this error, try `em explain {}`",
                log.id()
            );
            let mut display_list = DisplayList::from(snippet);
            display_list
                .body
                .push(DisplayLine::Raw(DisplayRawLine::Annotation {
                    annotation: annotate_snippets::display_list::Annotation {
                        annotation_type: DisplayAnnotationType::None,
                        id: None,
                        label: vec![DisplayTextFragment {
                            content: info_instruction,
                            style: DisplayTextStyle::Emphasis,
                        }],
                    },
                    source_aligned: false,
                    continuation: false,
                }));
            eprintln!("{}", display_list);
        } else {
            eprintln!("{}", DisplayList::from(snippet));
        }

        match msg_type {
            MessageType::Error => self.tot_errors += 1,
            MessageType::Warning => self.tot_warnings += 1,
            _ => {}
        }

        if let Some(max_errors) = self.max_errors {
            if self.tot_errors >= max_errors {
                return Err(EmblemError::too_many_errors(self.tot_errors));
            }
        }

        Ok(())
    }

    fn report(mut self) -> EmblemResult<()> {
        if self.verbosity() < Verbosity::Terse {
            return Ok(());
        }

        let tot_warnings = self.tot_warnings;
        if tot_warnings > 0 {
            let plural = if tot_warnings > 1 { "s" } else { "" };
            self.print(Log::warn(format!(
                "generated {} warning{plural}",
                tot_warnings
            )))?;
        }

        let tot_errors = self.tot_errors;
        if tot_errors == 0 {
            return Ok(());
        }
        let plural = if tot_errors > 1 { "s" } else { "" };
        let exe = std::env::current_exe().unwrap();
        let exe = exe
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        self.print(Log::error(format!(
            "`{exe}` failed due to {} error{plural}",
            tot_errors
        )))
    }
}

fn convert_message_type(msg_type: MessageType) -> AnnotationType {
    match msg_type {
        MessageType::Error => AnnotationType::Error,
        MessageType::Warning => AnnotationType::Warning,
        MessageType::Info => AnnotationType::Info,
        MessageType::Note => AnnotationType::Note,
        MessageType::Help => AnnotationType::Help,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use strum::IntoEnumIterator;

    #[test]
    fn problem_counters() {
        let error = Log::error("some error");
        let warn = Log::warn("some warning");
        let info = Log::info("some info");

        for verbosity in Verbosity::iter() {
            for colourise in [true, false] {
                println!("testing with verbosity {verbosity:?} (colourise: {colourise})");

                let expected_errors = 3;
                let expected_warnings = 3;
                let mut logger = PrettyLoggerBuilder::default()
                    .verbosity(verbosity)
                    .colourise(colourise)
                    .build()
                    .unwrap();
                for _ in 0..expected_errors {
                    logger.print(error.clone()).unwrap();
                }
                for _ in 0..expected_warnings {
                    logger.print(warn.clone()).unwrap();
                }
                logger.print(info.clone()).unwrap();
                assert_eq!(expected_errors, logger.tot_errors);
                assert_eq!(expected_warnings, logger.tot_warnings);

                logger.report().unwrap();
            }
        }
    }

    #[test]
    fn max_errors() {
        for verbosity in Verbosity::iter() {
            const ERROR_CAP: i32 = 3;
            let mut capped_logger = PrettyLogger::builder()
                .verbosity(verbosity)
                .max_errors(ERROR_CAP)
                .build()
                .unwrap();
            for i in 1..(1 + ERROR_CAP * 2) {
                let check_print_result = |msg_type: MessageType, result: EmblemResult<()>| {
                    if dbg!(dbg!(i) < ERROR_CAP) || dbg!(!verbosity.permits_printing(msg_type)) {
                        eprintln!("OKAY? {result:?}, {verbosity:?}");
                        result.unwrap()
                    } else {
                        eprintln!("ERR?  {result:?}, {verbosity:?}");
                        assert_eq!(
                            indoc::formatdoc!("run aborted after {i}"),
                            result.unwrap_err().to_string()
                        );
                    }
                };
                check_print_result(
                    MessageType::Error,
                    capped_logger.print(Log::error("this is bad")),
                );
                check_print_result(
                    MessageType::Warning,
                    capped_logger.print(Log::warn("this is concerning")),
                );
                check_print_result(
                    MessageType::Info,
                    capped_logger.print(Log::info("this is interesting")),
                );
            }

            let mut uncapped_logger = PrettyLogger::builder()
                .verbosity(verbosity)
                .build()
                .unwrap();
            for _ in 0..1000 {
                uncapped_logger
                    .print(Log::error("things keep going wrong"))
                    .unwrap();
            }
        }
    }
}
