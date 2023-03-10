pub mod messages;
mod verbosity;
mod note;
mod src;

pub use note::Note;
pub use src::Src;
pub use verbosity::Verbosity;

use self::messages::Message;
use annotate_snippets::{
    display_list::{
        DisplayAnnotationType, DisplayLine, DisplayList, DisplayRawLine, DisplayTextFragment,
        DisplayTextStyle, FormatOptions,
    },
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};

#[derive(Debug)]
pub struct LogArgs {
    /// Colourise log messages
    pub colour: bool,

    /// Make warnings into errors
    pub warnings_as_errors: bool,

    /// Output verbosity
    pub verbosity: Verbosity,
}

macro_rules! log_filter {
    ($name:ident, $verbosity:path) => {
        #[allow(clippy::crate_in_macro_def)]
        #[macro_export]
        macro_rules! $name {
            ($logger:expr, $msg:expr) => {
                if $logger.verbosity >= $verbosity {
                    #[allow(unused_imports)]
                    use crate::log::messages::Message;
                    $msg.log().print($logger)
                }
            };
        }
    };
}

log_filter!(alert, Verbosity::Terse);
log_filter!(inform, Verbosity::Verbose);
log_filter!(debug, Verbosity::Debug);

pub struct Logger {
    verbosity: Verbosity,
    colourise: bool,
    warnings_as_errors: bool,
    tot_errors: i32,
    tot_warnings: i32,
}

impl Logger {
    pub fn new(verbosity: Verbosity, colourise: bool, warnings_as_errors: bool) -> Self {
        Self {
            verbosity,
            colourise,
            warnings_as_errors,
            tot_errors: 0,
            tot_warnings: 0,
        }
    }

    pub fn report(mut self) {
        let tot_warnings = self.tot_warnings;
        let tot_errors = self.tot_errors;

        if tot_warnings > 0 {
            let plural = if tot_warnings > 1 { "s" } else { "" };
            alert!(
                &mut self,
                Log::warn(&format!("generated {} warning{plural}", tot_warnings))
            );
        }

        if tot_errors == 0 {
            return;
        }

        let plural = if tot_errors > 1 { "s" } else { "" };
        let exe = std::env::current_exe().unwrap();
        let exe = exe
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        alert!(
            &mut self,
            Log::error(&format!(
                "`{exe}` failed due to {} error{plural}",
                tot_errors
            ))
        );
    }
}

#[derive(Debug)]
pub struct Log<'i> {
    msg: String,
    msg_type: AnnotationType,
    id: Option<&'static str>,
    help: Option<String>,
    note: Option<String>,
    srcs: Vec<Src<'i>>,
    explainable: bool,
    expected: Option<Vec<String>>,
}

impl<'i> Log<'i> {
    fn new<S: Into<String>>(msg_type: AnnotationType, msg: S) -> Self {
        Self {
            msg: msg.into(),
            id: None,
            msg_type,
            help: None,
            note: None,
            srcs: Vec::new(),
            explainable: false,
            expected: None,
        }
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn print(self, logger: &mut Logger) {
        if !logger.verbosity.permits_printing(self.msg_type) {
            return;
        }

        let expected_string;
        let footer = {
            let mut footer = vec![];

            if let Some(ref help) = self.help {
                footer.push(Annotation {
                    id: None,
                    label: Some(help),
                    annotation_type: AnnotationType::Help,
                });
            }

            if let Some(ref note) = self.note {
                footer.push(Annotation {
                    id: None,
                    label: Some(note),
                    annotation_type: AnnotationType::Note,
                });
            }

            if let Some(ref expected) = self.expected {
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

        let snippet = Snippet {
            title: Some(Annotation {
                id: self.id,
                label: Some(&self.msg),
                annotation_type: match (logger.warnings_as_errors, self.msg_type) {
                    (true, AnnotationType::Warning) => AnnotationType::Error,
                    _ => self.msg_type,
                },
            }),
            slices: self
                .srcs
                .iter()
                .map(|s| {
                    let context = s.loc().context();
                    Slice {
                        source: context.src(),
                        line_start: s.loc().lines().0,
                        origin: Some(s.loc().file_name()),
                        fold: true,
                        annotations: s
                            .annotations()
                            .iter()
                            .map(|a| SourceAnnotation {
                                annotation_type: a.msg_type(),
                                label: &a.msg(),
                                range: a.loc().indices(&context),
                            })
                            .collect(),
                    }
                })
                .collect(),
            footer,
            opt: FormatOptions {
                color: logger.colourise,
                ..Default::default()
            },
        };

        if let Some(title) = &snippet.title {
            match title.annotation_type {
                AnnotationType::Error => logger.tot_errors += 1,
                AnnotationType::Warning => logger.tot_warnings += 1,
                _ => {}
            }
        }

        if self.explainable {
            if self.id.is_none() {
                panic!("internal error: explainable message has no id")
            }

            let info_instruction = &format!(
                "For more information about this error, try `em explain {}",
                self.id.unwrap()
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
    }

    pub fn error<S: Into<String>>(msg: S) -> Self {
        Self::new(AnnotationType::Error, msg)
    }

    pub fn warn<S: Into<String>>(msg: S) -> Self {
        Self::new(AnnotationType::Warning, msg)
    }

    #[allow(dead_code)]
    pub fn info<S: Into<String>>(msg: S) -> Self {
        Self::new(AnnotationType::Info, msg)
    }

    pub fn msg_type(&self) -> AnnotationType {
        self.msg_type
    }

    pub fn with_id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn id(&self) -> Option<&'static str> {
        self.id
    }

    pub fn explainable(mut self) -> Self {
        if self.id.is_none() {
            panic!("internal error: attempted to mark log without id as explainable")
        }

        self.explainable = true;
        self
    }

    pub fn is_explainable(&self) -> bool {
        self.explainable
    }

    pub fn with_note<S: Into<String>>(mut self, note: S) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn note(&self) -> &Option<String> {
        &self.note
    }

    pub fn with_help<S: Into<String>>(mut self, help: S) -> Self {
        assert!(self.help.is_none());

        self.help = Some(help.into());
        self
    }

    pub fn help(&self) -> &Option<String> {
        &self.help
    }

    pub fn with_src(mut self, src: Src<'i>) -> Self {
        self.srcs.push(src);
        self
    }

    pub fn srcs(&self) -> &Vec<Src<'i>> {
        &self.srcs
    }

    pub fn with_expected(mut self, expected: Vec<String>) -> Self {
        self.expected = Some(expected);
        self
    }

    pub fn expected(&self) -> &Option<Vec<String>> {
        &self.expected
    }

    pub fn successful(&self, warnings_as_errors: bool) -> bool {
        match self.msg_type {
            AnnotationType::Error => false,
            AnnotationType::Warning => !warnings_as_errors,
            _ => true,
        }
    }
}

#[cfg(test)]
impl Log<'_> {
    pub fn text(&self) -> Vec<&str> {
        let mut ret = vec![&self.msg[..]];

        for src in &self.srcs {
            ret.extend(src.text());
        }

        if let Some(h) = &self.help {
            ret.push(h);
        }

        ret
    }

    pub fn annotation_text(&self) -> Vec<String> {
        let mut ret = vec![self.msg.clone()];

        for src in &self.srcs {
            ret.extend(src.annotation_text());
        }

        if let Some(help) = &self.help {
            ret.push(help.clone());
        }

        ret
    }

    pub fn log_levels(&self) -> Vec<AnnotationType> {
        let mut ret = vec![self.msg_type];

        for src in &self.srcs {
            ret.extend(src.log_levels());
        }

        ret
    }

    pub fn assert_compliant(&self) {
        for text in self.text() {
            assert!(!text.is_empty(), "Got empty an message");

            let nchars = text.chars().count();
            assert!(nchars > 0, "Empty text in message {:?}", self);
            assert!(
                nchars < 60,
                "Message is not concise enough ({nchars} chars): {text} in {:?}",
                self
            );
            assert!(
                nchars >= 10,
                "Message is not long enough ({nchars} chars): {text} in {:?}",
                self
            );

            assert!(
                text.chars().filter(|c| *c == '\'').count() == 0,
                "Found dumb-quotes in message {:?}",
                text
            );

            assert!(
                text.chars().filter(|c| ['"', '“', '”'].contains(c)).count() == 0,
                "Found double quotes in message {:?} use directional single quotes only",
                text
            );

            let first_text_char = text
                .chars()
                .find(|c| !('0' <= *c && *c <= '9') && !['\'', '‘', '"', '“'].contains(c));
            assert!(
                first_text_char.is_some(),
                "Message contains no human-friendly text {:?}",
                text
            );
            assert!(
                first_text_char.unwrap().is_lowercase(),
                "Message does not start with lowercase: {:?} in {:?}",
                text,
                self
            );
        }

        for log_level in self.log_levels() {
            let ok = match self.msg_type {
                AnnotationType::Error => true,
                AnnotationType::Warning => log_level != AnnotationType::Error,
                AnnotationType::Info => {
                    [AnnotationType::Error, AnnotationType::Warning].contains(&log_level)
                }
                AnnotationType::Note | AnnotationType::Help => [
                    AnnotationType::Error,
                    AnnotationType::Warning,
                    AnnotationType::Info,
                ]
                .contains(&log_level),
            };
            assert!(
                ok,
                "Log level of sub-message ({:?}) exceeds parent ({:?})",
                log_level, self.msg_type
            );
        }
    }
}

impl<'i> Message<'i> for Log<'i> {
    fn log(self) -> Log<'i> {
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::{Location, Point};

    #[test]
    fn msg() {
        assert_eq!("hello, world!", Log::new(AnnotationType::Error, "hello, world!").msg(),);
    }

    #[test]
    fn id() {
        let id = "E69";
        assert_eq!(Some(id), Log::error("foo").with_id(id).id(),);
    }

    #[test]
    fn msg_type() {
        assert_eq!(AnnotationType::Error, Log::error("foo").msg_type());
        assert_eq!(AnnotationType::Warning, Log::warn("foo").msg_type());
        assert_eq!(AnnotationType::Info, Log::info("foo").msg_type());
    }

    #[test]
    fn is_explainable() {
        assert!(Log::error("foo")
            .with_id("E025")
            .explainable()
            .is_explainable());
        assert!(!Log::error("foo").is_explainable());
    }

    #[test]
    fn note() {
        let note = "william taylor".to_owned();
        assert_eq!(
            &Some(note.clone()),
            Log::error("foo").with_note(note.clone()).note()
        );
    }

    #[test]
    fn help() {
        let help = "is not coming".to_owned();
        assert_eq!(
            &Some(help.clone()),
            Log::error("foo").with_help(help.clone()).help()
        );
    }

    #[test]
    fn srcs() {
        let content = "hello, world";
        let srcs = [
            Point::new("main.em", content),
            Point::new("something-else.em", content),
        ]
        .into_iter()
        .map(|p| {
            let shifted = p.clone().shift("hello");
            Location::new(&p, &shifted)
        })
        .map(|l| Src::new(&l))
        .collect::<Vec<_>>();

        let mut log = Log::error("foo");
        for src in &srcs {
            log = log.with_src(src.clone());
        }

        assert_eq!(&srcs, log.srcs());
    }

    #[test]
    fn expected() {
        let expected = ["foo".into(), "bar".into()];
        assert_eq!(
            &Some(expected.to_vec()),
            Log::error("baz")
                .with_expected(expected.to_vec())
                .expected()
        );
    }

    #[test]
    fn successful() {
        for warnings_as_errors in [false, true] {
            assert!(!Log::error("foo").successful(warnings_as_errors));
            assert_eq!(
                Log::warn("foo").successful(warnings_as_errors),
                !warnings_as_errors
            );
            assert!(Log::info("foo").successful(warnings_as_errors));
        }
    }
}
