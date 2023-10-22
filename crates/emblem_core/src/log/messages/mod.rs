mod delimiter_mismatch;
mod empty_qualifier;
mod extra_comment_close;
mod heading_too_deep;
mod newline_in_attrs;
mod newline_in_emph_delimiter;
mod newline_in_inline_arg;
mod no_such_error_code;
mod too_many_qualifiers;
mod unclosed_comments;
mod unclosed_verbatim;
mod unexpected_char;
mod unexpected_eof;
mod unexpected_heading;
mod unexpected_token;

pub use delimiter_mismatch::DelimiterMismatch;
pub use empty_qualifier::EmptyQualifier;
pub use extra_comment_close::ExtraCommentClose;
pub use heading_too_deep::HeadingTooDeep;
pub use newline_in_attrs::NewlineInAttrs;
pub use newline_in_emph_delimiter::NewlineInEmphDelimiter;
pub use newline_in_inline_arg::NewlineInInlineArg;
pub use no_such_error_code::NoSuchErrorCode;
pub use too_many_qualifiers::TooManyQualifiers;
pub use unclosed_comments::UnclosedComments;
pub use unclosed_verbatim::UnclosedVerbatim;
pub use unexpected_char::UnexpectedChar;
pub use unexpected_eof::UnexpectedEOF;
pub use unexpected_heading::UnexpectedHeading;
pub use unexpected_token::UnexpectedToken;

use crate::log::Log;

use super::LogId;

pub trait Message {
    /// If implemented, returns the unique identifier for the message. This must have the form
    /// `Eddd` for digits `d`.
    fn id() -> LogId {
        Default::default()
    }

    /// Format this message into a log.
    fn log(self) -> Log;

    fn default() -> Box<Self>
    where
        Self: Default,
    {
        Default::default()
    }

    /// Explain the meaning of this error, why it usually comes up and if
    /// appropriate, how to avoid it.
    fn explain(&self) -> &'static str {
        ""
    }
}

pub struct MessageInfo {
    id: LogId,
    #[cfg(test)]
    default_log: Log,
    explanation: &'static str,
}

impl MessageInfo {
    pub fn id(&self) -> &LogId {
        &self.id
    }

    pub fn explanation(&self) -> &'static str {
        self.explanation
    }
}

pub fn messages() -> Vec<MessageInfo> {
    macro_rules! messages {
        ($($msg:ident),* $(,)?) => {
            {
                vec![
                    $(
                        {
                            let default = <$msg as Message>::default();
                            let explanation = default.explain();
                            MessageInfo {
                                id: $msg::id(),
                                #[cfg(test)]
                                default_log: default.log(),
                                explanation,
                            }
                        },
                    )*
                ]
            }
        };
    }

    messages![
        DelimiterMismatch,
        EmptyQualifier,
        ExtraCommentClose,
        HeadingTooDeep,
        NewlineInAttrs,
        NewlineInEmphDelimiter,
        NewlineInInlineArg,
        NoSuchErrorCode,
        TooManyQualifiers,
        UnclosedComments,
        UnclosedVerbatim,
        UnexpectedChar,
        UnexpectedEOF,
        UnexpectedHeading,
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
            for id in messages().iter().filter_map(|info| info.id.defined()) {
                assert!(seen.insert(id), "Non-unique id: {id}");
                assert!(RE.is_match(id), "Non-conformant id: {id}");
            }
        }

        #[test]
        fn log_application() {
            for (i, info) in messages().iter().enumerate() {
                assert_eq!(
                    &info.id,
                    info.default_log.id(),
                    "Incorrect id in log for {:?} (message type {})",
                    info.id,
                    i,
                );
            }
        }
    }

    #[test]
    fn text() {
        for info in messages() {
            info.default_log.assert_compliant()
        }
    }

    mod explanations {
        use super::*;

        #[test]
        fn prompt_offered_correctly() {
            for info in messages() {
                assert_eq!(
                    info.id.is_defined(),
                    info.default_log.is_explainable(),
                    "message id={:?} is {}explainable",
                    info.id,
                    if info.default_log.is_explainable() {
                        ""
                    } else {
                        "not "
                    }
                );
            }
        }

        #[test]
        fn not_too_long() {
            let mut failed = false;
            for info in messages() {
                let nchars = info.explanation.chars().count();
                let limit = if info.id.is_defined() { 1000 } else { 0 };

                if nchars > limit {
                    failed = true;
                    println!(
                        "{} explanation is too long: contains {nchars} chars (expected at most {limit})",
                        info.id.defined().unwrap(),
                    );
                }
            }
            assert!(!failed, "some explanations were too long, see above");
        }

        #[test]
        fn not_too_short() {
            let mut failed = false;

            for info in messages().iter().filter(|info| info.id.is_defined()) {
                let nchars = info.explanation.chars().count();
                if nchars < 100 {
                    failed = true;
                    println!(
                        "{} explanation is too short: contains {} chars",
                        info.id.defined().unwrap(),
                        nchars
                    );
                }
            }
            assert!(!failed, "some explanations were too short, see above");
        }

        #[test]
        fn lines_not_too_long() {
            const LINE_MAX_LEN: usize = 90;
            let mut failed = false;

            for info in messages().iter().filter(|info| !info.id.is_defined()) {
                for line in info.explanation().lines() {
                    if line.chars().count() > LINE_MAX_LEN {
                        failed = true;
                        println!(
                            "{}: Line longer than {LINE_MAX_LEN} chars: {line:?}",
                            info.id().defined().unwrap()
                        );
                        println!("\tline should end before the inserted `|`:");
                        println!("\t{}|{}", &line[..81], &line[81..]);
                    }
                }
            }

            if failed {
                panic!("some lines were too long");
            }
        }
    }
}
