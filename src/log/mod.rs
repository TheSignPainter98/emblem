pub mod messages;

use std::process::ExitCode;

use self::messages::Message;
use crate::{
    args::{LogArgs, Verbosity},
    parser::Location,
};
use annotate_snippets::{
    display_list::{
        DisplayAnnotationType, DisplayLine, DisplayList, DisplayRawLine, DisplayTextFragment,
        DisplayTextStyle, FormatOptions,
    },
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use parking_lot::Mutex;

pub static mut VERBOSITY: Verbosity = Verbosity::Terse;

static mut COLOURISE: bool = true;
static mut WARNINGS_AS_ERRORS: bool = false;
static mut TOT_ERRORS: Mutex<i32> = Mutex::new(0);
static mut TOT_WARNINGS: Mutex<i32> = Mutex::new(0);

macro_rules! logger {
    ($name:ident, $verbosity:ident) => {
        #[allow(clippy::crate_in_macro_def)]
        #[macro_export]
        macro_rules! $name {
            ($msg:expr) => {
                if unsafe { crate::log::VERBOSITY } >= crate::args::Verbosity::$verbosity {
                    #[allow(unused_imports)]
                    use crate::log::messages::Message;
                    $msg.log().print()
                }
            };
        }
    };
}

logger!(alert, Terse);
logger!(inform, Verbose);
logger!(debug, Debug);

pub fn init(args: LogArgs) {
    unsafe {
        COLOURISE = args.colour;
        WARNINGS_AS_ERRORS = args.warnings_as_errors;
        VERBOSITY = args.verbosity;
    }
}

pub fn report() -> ExitCode {
    let tot_warnings = unsafe { *TOT_WARNINGS.lock() };
    let tot_errors = unsafe { *TOT_ERRORS.lock() };

    if tot_warnings > 0 {
        let plural = if tot_warnings > 1 { "s" } else { "" };
        alert!(Log::warn(&format!(
            "generated {tot_warnings} warning{plural}"
        )));
    }

    if tot_errors > 0 {
        let plural = if tot_errors > 1 { "s" } else { "" };
        let exe = std::env::current_exe().unwrap();
        let exe = exe
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        alert!(Log::error(&format!(
            "`{exe}` failed due to {tot_errors} error{plural}"
        )));
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

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

    pub fn print(self) {
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
                annotation_type: match (unsafe { WARNINGS_AS_ERRORS }, self.msg_type) {
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
                color: unsafe { COLOURISE },
                ..Default::default()
            },
        };

        if let Some(title) = &snippet.title {
            match title.annotation_type {
                AnnotationType::Error => unsafe { *TOT_ERRORS.lock() += 1 },
                AnnotationType::Warning => unsafe { *TOT_WARNINGS.lock() += 1 },
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

    pub fn id(mut self, id: &'static str) -> Self {
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

    #[allow(dead_code)]
    pub fn note<S: Into<String>>(mut self, note: S) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn help<S: Into<String>>(mut self, help: S) -> Self {
        assert!(self.help.is_none());

        self.help = Some(help.into());
        self
    }

    pub fn src(mut self, src: Src<'i>) -> Self {
        self.srcs.push(src);
        self
    }

    pub fn expect_one_of(self, expected: &Vec<String>) -> Self {
        let len = expected.len();
        if len == 1 {
            return self.note(format!("expected {}", expected[0]));
        }

        let mut pretty_expected = Vec::new();
        for (i, e) in expected.iter().enumerate() {
            if i > 0 {
                pretty_expected.push(if i < len - 1 { ", " } else { " or " })
            }
            pretty_expected.push(e);
        }

        self.note(format!("expected one of {}", pretty_expected.concat()))
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

    pub fn test(&self) {
        for text in self.get_annotation_text() {
            assert!(!text.is_empty(), "Got empty an message");
            assert!(
                text.chars().next().unwrap().is_lowercase(),
                "Message does not start with lowercase: {:?}",
                text
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
