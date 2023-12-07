mod batch_logger;
pub mod messages;
mod note;
mod src;
mod verbosity;

use std::{borrow::Cow, fmt::Display};

use crate::{lint::LintId, Result};

pub use self::batch_logger::BatchLogger;
pub use self::messages::Message;
pub use self::note::Note;
pub use self::src::Src;
pub use self::verbosity::Verbosity;

pub trait Logger: Sized {
    fn verbosity(&self) -> Verbosity;
    fn print(&mut self, log: Log) -> Result<()>;
    fn report(self) -> Result<()>;
}

#[macro_export]
macro_rules! log_error {
    ($ctx:expr) => {
        std::compile_error!("log_error requires a message to log");
    };
    ($ctx:expr, $($arg:tt)+) => {
        $crate::log::__log!($ctx, $crate::Verbosity::Terse, $crate::Log::error, $($arg)+)
    };
}

#[macro_export]
macro_rules! log_warning {
    ($ctx:expr) => {
        std::compile_error!("log_warning requires a message to log");
    };
    ($ctx:expr, $($arg:tt)+) => {
        $crate::log::__log!($ctx, $crate::Verbosity::Terse, $crate::Log::warning, $($arg)+)
    };
}

#[macro_export]
macro_rules! log_info {
    ($ctx:expr) => {
        std::compile_error!("log_info requires a message to log");
    };
    ($ctx:expr, $($arg:tt)+) => {
        $crate::log::__log!($ctx, $crate::Verbosity::Verbose, $crate::Log::info, $($arg)+)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($ctx:expr) => {
        std::compile_error!("log_debug requires a message to log");
    };
    ($ctx:expr, $($arg:tt)+) => {
        $crate::log::__log!($ctx, $crate::Verbosity::Debug, $crate::Log::debug, $($arg)+)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __log {
    ($ctx:expr, $max_lvl:path, $log_constructor:path, $($arg:tt)*) => {{
        let ctx: &$crate::Context<_> = $ctx;
        if ctx.verbosity() >= $max_lvl {
            ctx.print($log_constructor(std::format!("{}", std::format_args!($($arg)*))))
        } else {
            Ok(())
        }
    }};
}
#[allow(unused_imports)]
pub use __log;

#[derive(Clone, Debug, PartialEq)]
pub struct Log {
    pub(crate) msg: String,
    pub(crate) msg_type: MessageType,
    pub(crate) id: LogId,
    pub(crate) help: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) srcs: Vec<Src>,
    pub(crate) explainable: bool,
    pub(crate) expected: Option<Vec<String>>,
}

impl Log {
    fn new(msg_type: MessageType, msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            id: LogId::Undefined,
            msg_type,
            help: None,
            note: None,
            srcs: Vec::new(),
            explainable: false,
            expected: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self::new(MessageType::Error, msg)
    }

    pub fn warning(msg: impl Into<String>) -> Self {
        Self::new(MessageType::Warning, msg)
    }

    #[allow(dead_code)]
    pub fn info(msg: impl Into<String>) -> Self {
        Self::new(MessageType::Info, msg)
    }

    #[allow(dead_code)]
    pub fn debug(msg: impl Into<String>) -> Self {
        Self::new(MessageType::Info, msg)
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn msg_type(&self) -> MessageType {
        self.msg_type
    }

    pub fn with_id(mut self, id: LogId) -> Self {
        self.id = id;
        self
    }

    pub fn id(&self) -> &LogId {
        &self.id
    }

    pub fn explainable(mut self) -> Self {
        if !self.id.is_defined() {
            panic!("internal error: attempted to mark log without id as explainable")
        }

        self.explainable = true;
        self
    }

    pub fn is_explainable(&self) -> bool {
        self.explainable
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn note(&self) -> &Option<String> {
        &self.note
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        assert!(self.help.is_none());

        self.help = Some(help.into());
        self
    }

    pub fn help(&self) -> &Option<String> {
        &self.help
    }

    pub fn with_src(mut self, src: Src) -> Self {
        self.srcs.push(src);
        self
    }

    pub fn srcs(&self) -> &Vec<Src> {
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
            MessageType::Error => false,
            MessageType::Warning => !warnings_as_errors,
            _ => true,
        }
    }
}

#[cfg(test)]
impl Log {
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

    pub fn message_types(&self) -> Vec<MessageType> {
        let mut ret = vec![self.msg_type];

        for src in &self.srcs {
            ret.extend(src.message_types());
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
                .find(|c| !('0' <= *c && *c <= '9' || ['\'', '‘', '"', '“'].contains(c)));
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

        for msg_type in self.message_types() {
            let ok = match self.msg_type {
                MessageType::Error => true,
                MessageType::Warning => msg_type != MessageType::Error,
                MessageType::Info => [MessageType::Error, MessageType::Warning].contains(&msg_type),
                MessageType::Note | MessageType::Help => {
                    [MessageType::Error, MessageType::Warning, MessageType::Info]
                        .contains(&msg_type)
                }
            };
            assert!(
                ok,
                "Log level of sub-message ({:?}) exceeds parent ({:?})",
                msg_type, self.msg_type
            );
        }
    }
}

impl Message for Log {
    fn log(self) -> Log {
        self
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MessageType {
    Error,
    Warning,
    Info,
    Note,
    Help,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub enum LogId {
    #[default]
    Undefined,
    Defined(Cow<'static, str>),
}

impl LogId {
    pub fn defined(&self) -> Option<&str> {
        match self {
            Self::Defined(raw) => Some(raw),
            _ => None,
        }
    }

    pub fn is_defined(&self) -> bool {
        matches!(self, Self::Defined(_))
    }
}

impl From<&'static str> for LogId {
    fn from(raw: &'static str) -> Self {
        Self::Defined(raw.into())
    }
}

impl From<String> for LogId {
    fn from(raw: String) -> Self {
        Self::Defined(raw.into())
    }
}

impl From<LintId> for LogId {
    fn from(id: LintId) -> Self {
        id.raw().into()
    }
}

impl From<LogId> for Option<&'static str> {
    fn from(id: LogId) -> Self {
        match id {
            LogId::Defined(Cow::Borrowed(raw)) => Some(raw),
            _ => None,
        }
    }
}

impl Display for LogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defined(raw) => raw.fmt(f),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    use strum::IntoEnumIterator;

    use super::*;
    use crate::{
        parser::{Location, Point},
        Context,
    };

    #[test]
    fn filters() {
        struct Test<F: Fn(&Context<BatchLogger>) -> Result<()>> {
            name: Cow<'static, str>,
            printing_verbosities: &'static [Verbosity],
            func: Option<F>,
        }

        impl<F: Fn(&Context<BatchLogger>) -> Result<()>> Test<F> {
            fn new(name: impl Into<Cow<'static, str>>) -> Self {
                Self {
                    name: name.into(),
                    printing_verbosities: &[],
                    func: None,
                }
            }

            fn at_verbosities(mut self, printing_verbosities: &'static [Verbosity]) -> Self {
                self.printing_verbosities = printing_verbosities;
                self
            }

            fn func(mut self, func: F) -> Self {
                self.func = Some(func);
                self
            }

            fn produces_log(self, expected_log: Log) {
                println!("testing {}...", self.name);

                let Some(func) = self.func else {
                    panic!("{}: test has no func to run!", self.name)
                };

                for verbosity in Verbosity::iter() {
                    let logger = BatchLogger::new(verbosity);
                    let ctx = Context::new(logger);
                    func(&ctx).unwrap();
                    if self.printing_verbosities.contains(&verbosity) {
                        assert_eq!(ctx.logger().logs(), [expected_log.clone()])
                    } else {
                        assert_eq!(ctx.logger().logs(), []);
                    }
                }
            }
        }

        Test::new("error")
            .at_verbosities(&[Verbosity::Terse, Verbosity::Verbose, Verbosity::Debug])
            .func(|ctx| log_error!(ctx, "oh {}!", "no"))
            .produces_log(Log::error("oh no!"));
        Test::new("warning")
            .at_verbosities(&[Verbosity::Terse, Verbosity::Verbose, Verbosity::Debug])
            .func(|ctx| log_warning!(ctx, "oh {}!", "no"))
            .produces_log(Log::warning("oh no!"));
        Test::new("info")
            .at_verbosities(&[Verbosity::Verbose, Verbosity::Debug])
            .func(|ctx| log_info!(ctx, "oh {}!", "no"))
            .produces_log(Log::info("oh no!"));
        Test::new("debug")
            .at_verbosities(&[Verbosity::Debug])
            .func(|ctx| log_debug!(ctx, "oh {}!", "no"))
            .produces_log(Log::debug("oh no!"));
    }

    #[test]
    fn msg() {
        assert_eq!(
            "hello, world!",
            Log::new(MessageType::Error, "hello, world!").msg(),
        );
    }

    #[test]
    fn id() {
        let id = LogId::from("E69");
        assert_eq!(id, *Log::error("foo").with_id(id.clone()).id());
    }

    #[test]
    fn msg_type() {
        assert_eq!(MessageType::Error, Log::error("foo").msg_type());
        assert_eq!(MessageType::Warning, Log::warning("foo").msg_type());
        assert_eq!(MessageType::Info, Log::info("foo").msg_type());
    }

    #[test]
    fn is_explainable() {
        assert!(Log::error("foo")
            .with_id("E025".into())
            .explainable()
            .is_explainable());
        assert!(!Log::error("foo").is_explainable());
    }

    #[test]
    fn note() {
        let note = "william taylor".to_owned();
        assert_eq!(
            &Some(note.clone()),
            Log::error("foo").with_note(note).note()
        );
    }

    #[test]
    fn help() {
        let help = "is not coming".to_owned();
        assert_eq!(
            &Some(help.clone()),
            Log::error("foo").with_help(help).help()
        );
    }

    #[test]
    fn srcs() {
        let ctx = Context::test_new();
        let content = ctx.alloc_file_content("hello, world");
        let srcs = [
            Point::at_start_of(ctx.alloc_file_name("main.em"), content.clone()),
            Point::at_start_of(ctx.alloc_file_name("something-else.em"), content),
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
                Log::warning("foo").successful(warnings_as_errors),
                !warnings_as_errors
            );
            assert!(Log::info("foo").successful(warnings_as_errors));
        }
    }

    #[test]
    fn log_id() {
        let undefined = LogId::Undefined;
        assert!(!undefined.is_defined());
        assert_eq!(None, undefined.defined());

        let raw_id = "E025";
        let defined = LogId::from(raw_id);
        assert_eq!(LogId::Defined(raw_id.into()), defined);
        assert!(defined.is_defined());
        assert_eq!(Some(raw_id), defined.defined());
    }
}
