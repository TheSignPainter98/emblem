use crate::{
    args::{LogArgs, Verbosity},
    parser::Location,
};
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use parking_lot::Mutex;
use std::error::Error;

pub static mut VERBOSITY: Verbosity = Verbosity::Terse;

static mut COLOURISE: bool = true;
static mut FATAL_WARNINGS: bool = false;
static mut TOT_ERRS: Mutex<i32> = Mutex::new(0);

pub fn init(args: LogArgs) {
    unsafe {
        COLOURISE = args.colour;
        FATAL_WARNINGS = args.fatal_warnings;
        VERBOSITY = args.verbosity;
    }
}

macro_rules! logger {
    ($name:ident, $verbosity:path, $VERBOSITY:path) => {
        macro_rules! $name {
            ($msg:expr) => {
                use crate::args::Verbosity;
                use annotate_snippets::{display_list::DisplayList, snippet::Snippet};
                if unsafe { $VERBOSITY } >= $verbosity {
                    log(DisplayList::from(Snippet::from($msg.into())));
                }
            };
        }
    };
}

// logger!(fatal, Verbosity::Terse, crate::log::VERBOSITY);
logger!(alert, Verbosity::Terse, crate::log::VERBOSITY);
logger!(warn, Verbosity::Terse, crate::log::VERBOSITY);
logger!(inform, Verbosity::Verbose, crate::log::VERBOSITY);
logger!(debug, Verbosity::Debug, crate::log::VERBOSITY);

fn log<'i>(msg: impl Loggable<'i>) {
    println!("{}", DisplayList::from(msg.snippet()));
}

trait Loggable<'i> where Self: Sized {
    fn snippet(&self) -> Snippet<'i>;
}

pub struct Log<'i> {
    msg: &'i str,
    msg_type: AnnotationType,
    id: Option<&'static str>,
    help: Option<Msg<'i>>,
    srcs: Vec<Src<'i>>,
}

impl<'i> Log<'i> {
    fn new(msg_type: AnnotationType, msg: &'i str) -> Self {
        Self {
            msg: msg.into(),
            id: None,
            msg_type,
            help: None,
            srcs: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn error(msg: &'i str) -> Self {
        Self::new(AnnotationType::Error, msg)
    }

    #[allow(dead_code)]
    pub fn warn(msg: &'i str) -> Self {
        Self::new(AnnotationType::Warning, msg)
    }

    #[allow(dead_code)]
    pub fn info(msg: &'i str) -> Self {
        Self::new(AnnotationType::Info, msg)
    }

    #[allow(dead_code)]
    pub fn note(msg: &'i str) -> Self {
        Self::new(AnnotationType::Note, msg)
    }

    #[allow(dead_code)]
    pub fn id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
        self
    }

    #[allow(dead_code)]
    pub fn help(mut self, help: Msg<'i>) -> Self {
        self.help = Some(help);
        self
    }

    #[allow(dead_code)]
    pub fn src(mut self, src: Src<'i>) -> Self {
        self.srcs.push(src);
        self
    }
}

impl<'i> Into<Snippet<'i>> for Log<'i> {
    fn into(self) -> Snippet<'i> {
        Snippet {
            title: Some(Annotation {
                id: self.id,
                label: Some(self.msg),
                annotation_type: self.msg_type,
            }),
            slices: vec![],
            footer: vec![],
            opt: FormatOptions {
                color: unsafe { COLOURISE },
                ..Default::default()
            },
        }
    }
}

pub struct Msg<'i> {
    loc: Option<Location<'i>>,
    msg: String,
    msg_type: AnnotationType,
}

impl<'i> Msg<'i> {
    fn new<S: Into<String>>(msg_type: AnnotationType, msg: S) -> Self {
        Self {
            loc: None,
            msg: msg.into(),
            msg_type,
        }
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
    pub fn loc(mut self, loc: &Location<'i>) -> Self {
        self.loc = Some(loc.clone());
        self
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
