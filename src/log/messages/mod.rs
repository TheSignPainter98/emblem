mod extra_comment_close;
mod newline_in_inline_arg;
mod no_such_error_code;
mod unclosed_comments;
mod unexpected_char;
mod unexpected_eof;
mod unexpected_token;

pub use extra_comment_close::ExtraCommentClose;
pub use newline_in_inline_arg::NewlineInInlineArg;
pub use no_such_error_code::NoSuchErrorCode;
pub use unclosed_comments::UnclosedComments;
pub use unexpected_char::UnexpectedChar;
pub use unexpected_eof::UnexpectedEOF;
pub use unexpected_token::UnexpectedToken;

use crate::log::Log;

pub trait Message<'i> {
    /// If implemented, returns the unique identifier for the message. This must have the form
    /// `Eddd` for digits `d`.
    fn id() -> &'static str
    where
        Self: Sized,
    {
        ""
    }

    /// Format this message into a log.
    fn log(self) -> Log<'i>;

    fn default() -> Box<Self>
    where
        Self: Default,
    {
        Default::default()
    }

    /// Explain the meaning of this error, why it usually comes up and if
    /// appropriate, how to avoid it.
    fn explain() -> &'static str
    where
        Self: Sized,
    {
        // TODO(kcza): remove default empty implementation
        ""
    }
}

pub struct MessageInfo {
    id: &'static str,
    #[cfg(test)]
    default_log: Log<'static>,
    explanation: &'static str,
}

impl MessageInfo {
    pub fn id(&self) -> &'static str {
        self.id
    }

    pub fn explanation(&self) -> &'static str {
        self.explanation
    }
}

pub fn messages() -> Vec<MessageInfo> {
    macro_rules! messages {
        ($($msg:ident),* $(,)?) => {
            {
                let mut ret = Vec::new();
                $(
                    ret.push(MessageInfo {
                        id: $msg::id(),
                        #[cfg(test)]
                        default_log: <$msg as Message<'_>>::default().log(),
                        explanation: $msg::explain()
                    });
                )*
                ret
            }
        };
    }

    messages![
        ExtraCommentClose,
        NewlineInInlineArg,
        NoSuchErrorCode,
        UnclosedComments,
        UnexpectedChar,
        UnexpectedEOF,
        UnexpectedToken,
    ]
}

#[cfg(test)]
mod test {
    use super::*;

    mod ids {
        use super::*;
        use lazy_static::lazy_static;
        use regex::Regex;
        use std::collections::HashSet;

        #[test]
        fn naming() {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^E\d{3}$").unwrap();
            }

            let mut seen = HashSet::new();
            for info in messages().iter().filter(|info| !info.id.is_empty()) {
                assert!(seen.insert(info.id), "Non-unique id: {}", info.id);
                assert!(RE.is_match(info.id), "Non-conformant id: {}", info.id);
            }
        }

        #[test]
        fn log_application() {
            for (i, info) in messages().iter().enumerate() {
                let id = match info.id {
                    "" => None,
                    s => Some(s),
                };
                let log = &info.default_log;
                assert_eq!(
                    id,
                    log.get_id(),
                    "Incorrect id in log for {:?} (message type {})",
                    info.id,
                    i,
                );
            }
        }
    }

    #[test]
    fn text() {
        for (i, info) in messages().iter().enumerate() {
            for text in info.default_log.get_text() {
                let nchars = text.chars().count();

                assert!(nchars > 0, "Empty text in message of type {i}");
                assert!(
                    nchars < 60,
                    "Default message is not concise enough ({nchars} chars): {text}",
                );

                assert!(
                    text.chars().next().unwrap().is_lowercase(),
                    "Does not start with a lowercase character: {}",
                    text
                );
            }
        }
    }

    mod explanations {
        use super::*;

        #[test]
        fn not_too_long() {
            let mut failed = false;
            for info in messages() {
                let nchars = info.explanation.chars().count();
                let limit = if info.id.is_empty() { 0 } else { 1000 };

                if nchars > limit {
                    failed = true;
                    println!(
                        "{} explanation is too long: contains {} chars",
                        info.id, nchars
                    );
                }
            }
            assert!(!failed, "some explanations were too long, see above");
        }

        #[test]
        fn not_too_short() {
            let mut failed = false;

            for info in messages().iter().filter(|info| !info.id.is_empty()) {
                let nchars = info.explanation.chars().count();
                if nchars < 100 {
                    failed = true;
                    println!(
                        "{} explanation is too short: contains {} chars",
                        info.id, nchars
                    );
                }
            }
            assert!(!failed, "some explanations were too short, see above");
        }
    }
}
