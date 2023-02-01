pub mod messages;

use std::process::ExitCode;

use self::messages::Message;
use crate::{
    args::{LogArgs, Verbosity},
    parser::Location,
};
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
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
                    $msg.log().print();
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
    srcs: Vec<Src<'i>>,
}

impl<'i> Log<'i> {
    fn new<S: Into<String>>(msg_type: AnnotationType, msg: S) -> Self {
        Self {
            msg: msg.into(),
            id: None,
            msg_type,
            help: None,
            srcs: Vec::new(),
        }
    }

    pub fn print(self) {
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
                .map(|s| Slice {
                    source: s.loc.src(),
                    line_start: 1,
                    origin: Some(s.loc.file_name()),
                    fold: true,
                    annotations: s
                        .annotations
                        .iter()
                        .map(|a| SourceAnnotation {
                            annotation_type: a.msg_type,
                            label: &a.msg,
                            range: a.loc.indices(),
                        })
                        .collect(),
                })
                .collect(),
            footer: self
                .help
                .iter()
                .map(|help| Annotation {
                    id: None,
                    label: Some(help),
                    annotation_type: AnnotationType::Note,
                })
                .collect(),
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

        eprintln!("{}", DisplayList::from(snippet));
    }

    #[allow(dead_code)]
    pub fn error<S: Into<String>>(msg: S) -> Self {
        Self::new(AnnotationType::Error, msg)
    }

    #[allow(dead_code)]
    pub fn warn<S: Into<String>>(msg: S) -> Self {
        Self::new(AnnotationType::Warning, msg)
    }

    #[allow(dead_code)]
    pub fn info<S: Into<String>>(msg: S) -> Self {
        Self::new(AnnotationType::Info, msg)
    }

    #[allow(dead_code)]
    pub fn note<S: Into<String>>(msg: S) -> Self {
        Self::new(AnnotationType::Note, msg)
    }

    #[allow(dead_code)]
    pub fn id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
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
        // TODO(kcza): replace with iter.intersperse once stable.
        let len = expected.len();
        let mut pretty_expected = Vec::new();
        for (i, e) in expected.iter().enumerate() {
            if i > 0 {
                pretty_expected.push(if i < len - 1 { ", " } else { " or " })
            }
            pretty_expected.push(e);
        }

        self.help(format!("expected one of {}", pretty_expected.concat()))
    }
}

#[cfg(test)]
impl Log<'_> {
    pub fn get_id(&self) -> Option<&str> {
        self.id
    }
}

impl<'i> Message<'i> for Log<'i> {
    fn log(self) -> Log<'i> {
        self
    }
}

pub struct Msg<'i> {
    loc: Location<'i>,
    msg: String,
    msg_type: AnnotationType,
}

impl<'i> Msg<'i> {
    fn new<S: Into<String>>(msg_type: AnnotationType, loc: &Location<'i>, msg: S) -> Self {
        Self {
            loc: loc.clone(),
            msg: msg.into(),
            msg_type,
        }
    }

    #[allow(dead_code)]
    pub fn error<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Error, loc, msg)
    }

    #[allow(dead_code)]
    pub fn warn<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Warning, loc, msg)
    }

    #[allow(dead_code)]
    pub fn info<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Info, loc, msg)
    }

    #[allow(dead_code)]
    pub fn note<S: Into<String>>(loc: &Location<'i>, msg: S) -> Self {
        Self::new(AnnotationType::Note, loc, msg)
    }
}

pub struct Src<'i> {
    loc: Location<'i>,
    annotations: Vec<Msg<'i>>,
}

impl<'i> Src<'i> {
    #[allow(dead_code)]
    pub fn new(loc: &Location<'i>) -> Self {
        Self {
            loc: loc.clone(),
            annotations: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn annotate(mut self, annotation: Msg<'i>) -> Self {
        self.annotations.push(annotation);
        self
    }
}
