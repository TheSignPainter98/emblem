pub mod messages;

use self::messages::Message;
use crate::parser::Location;
use annotate_snippets::{
    display_list::{
        DisplayAnnotationType, DisplayLine, DisplayList, DisplayRawLine, DisplayTextFragment,
        DisplayTextStyle, FormatOptions,
    },
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verbosity {
    Terse,
    Verbose,
    Debug,
}

impl Verbosity {
    pub fn permits_printing(&self, msg_type: AnnotationType) -> bool {
        match (self, msg_type) {
            (Self::Terse, AnnotationType::Error) | (Self::Terse, AnnotationType::Warning) => true,
            (Self::Terse, _) => false,
            _ => true,
        }
    }
}

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
        }
    }

    pub fn print(self, logger: &mut Logger) {
        if !logger.verbosity.permits_printing(self.msg_type) {
            return;
        }

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
                    let context = s.loc.context();
                    Slice {
                        source: context.src(),
                        line_start: s.loc.lines().0,
                        origin: Some(s.loc.file_name()),
                        fold: true,
                        annotations: s
                            .annotations
                            .iter()
                            .map(|a| SourceAnnotation {
                                annotation_type: a.msg_type,
                                label: &a.msg,
                                range: a.loc.indices(&context),
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

    pub fn with_id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn explainable(mut self) -> Self {
        if self.id.is_none() {
            panic!("internal error: attempted to mark log without id as explainable")
        }

        self.explainable = true;
        self
    }

    pub fn with_note<S: Into<String>>(mut self, note: S) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn with_help<S: Into<String>>(mut self, help: S) -> Self {
        assert!(self.help.is_none());

        self.help = Some(help.into());
        self
    }

    pub fn with_src(mut self, src: Src<'i>) -> Self {
        self.srcs.push(src);
        self
    }

    pub fn with_expectation_note(self, expected: &Vec<String>) -> Self {
        let len = expected.len();
        if len == 1 {
            return self.with_note(format!("expected {}", expected[0]));
        }

        let mut pretty_expected = Vec::new();
        for (i, e) in expected.iter().enumerate() {
            if i > 0 {
                pretty_expected.push(if i < len - 1 { ", " } else { " or " })
            }
            pretty_expected.push(e);
        }

        self.with_note(format!("expected one of {}", pretty_expected.concat()))
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
    pub fn get_id(&self) -> Option<&str> {
        self.id
    }

    pub fn get_text(&self) -> Vec<&str> {
        let mut ret = vec![&self.msg[..]];

        for src in &self.srcs {
            ret.extend(src.get_text());
        }

        if let Some(h) = &self.help {
            ret.push(h);
        }

        ret
    }

    pub fn get_annotation_text(&self) -> Vec<String> {
        let mut ret = vec![self.msg.clone()];

        for src in &self.srcs {
            ret.extend(src.get_annotation_text());
        }

        if let Some(help) = &self.help {
            ret.push(help.clone());
        }

        ret
    }

    pub fn get_log_levels(&self) -> Vec<AnnotationType> {
        let mut ret = vec![self.msg_type];

        for src in &self.srcs {
            ret.extend(src.get_log_levels());
        }

        ret
    }

    pub fn is_explainable(&self) -> bool {
        self.explainable
    }

    pub fn assert_compliant(&self) {
        for text in self.get_text() {
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

        for log_level in self.get_log_levels() {
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

#[derive(Debug)]
pub struct Note<'i> {
    loc: Location<'i>,
    msg: String,
    msg_type: AnnotationType,
}

impl<'i> Note<'i> {
    fn new<S: Into<String>>(msg_type: AnnotationType, loc: &Location<'i>, msg: S) -> Self {
        Self {
            loc: loc.clone(),
            msg: msg.into(),
            msg_type,
        }
    }

    pub fn error<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Error, loc, msg)
    }

    #[allow(dead_code)]
    pub fn warn<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Warning, loc, msg)
    }

    pub fn info<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Info, loc, msg)
    }

    #[allow(dead_code)]
    pub fn help<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Help, loc, msg)
    }
}

#[cfg(test)]
impl Note<'_> {
    fn get_text(&self) -> Vec<&str> {
        vec![&self.msg]
    }

    fn get_annotation_text(&self) -> Vec<String> {
        vec![format!("{}: {}", self.loc, self.msg)]
    }

    fn get_log_levels(&self) -> Vec<AnnotationType> {
        vec![self.msg_type]
    }
}

#[derive(Debug)]
pub struct Src<'i> {
    loc: Location<'i>,
    annotations: Vec<Note<'i>>,
}

impl<'i> Src<'i> {
    pub fn new(loc: &Location<'i>) -> Self {
        Self {
            loc: loc.clone(),
            annotations: Vec::new(),
        }
    }

    pub fn annotate(mut self, note: Note<'i>) -> Self {
        self.annotations.push(note);
        self
    }
}

#[cfg(test)]
impl Src<'_> {
    fn get_text(&self) -> Vec<&str> {
        self.annotations.iter().flat_map(|a| a.get_text()).collect()
    }

    fn get_annotation_text(&self) -> Vec<String> {
        self.annotations
            .iter()
            .flat_map(|a| a.get_annotation_text())
            .collect()
    }

    fn get_log_levels(&self) -> Vec<AnnotationType> {
        self.annotations
            .iter()
            .flat_map(|a| a.get_log_levels())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
