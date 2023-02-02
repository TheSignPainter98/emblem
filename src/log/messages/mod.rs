mod newline_in_inline_arg;
mod unexpected_eof;
mod unexpected_token;

pub use newline_in_inline_arg::NewlineInInlineArg;
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

#[cfg(test)]
pub struct MessageInfo {
    id: &'static str,
    default_log: Log<'static>,
    explanation: &'static str,
}

#[cfg(test)]
fn messages() -> Vec<MessageInfo> {
    macro_rules! messages {
        ($($msg:ident),*) => {
            {
                let mut ret = Vec::new();
                $(
                    ret.push(MessageInfo {
                        id: $msg::id(),
                        default_log: <$msg as Message<'_>>::default().log(),
                        explanation: $msg::explain()
                    });
                )*
                ret
            }
        };
    }

    messages![UnexpectedEOF, UnexpectedToken, NewlineInInlineArg]
}

// #[cfg(test)]
// fn defaults<'i>() -> Vec<(&'static str, Box<dyn Message<'i>>)> {
//     macro_rules! defaults {
//         ($($msg:ident),*) => {
//             {
//                 let mut ret = Vec::new();
//                 $(
//                     ret.push(($msg::id(), <$msg as Message>::default()));
//                 )*
//                 ret
//             }
//         };
//     }

//     defaults![UnexpectedEOF]
// }

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
            for info in messages() {
                let id = info.id;
                let log = Box::new(info.default_log);
                assert_eq!(
                    id,
                    log.get_id().unwrap_or(""),
                    "Incorrect id in log for {}",
                    id
                );
            }
        }
    }

    mod descriptions {
        use super::*;

        #[test]
        fn not_too_long() {
            for info in messages() {
                let nchars = info.explanation.chars().count();
                assert!(
                    nchars <= 1000,
                    "{} explanation is too long: contains {} chars",
                    info.id,
                    nchars
                );
            }
        }

        //         #[test]
        //         fn not_too_short() {
        //             for info in messages() {
        //                 let nchars = info.explanation.chars().count();
        //                 assert!(
        //                     nchars >= 100,
        //                     "{} description is too short: contains {} chars",
        //                     info.id,
        //                     nchars
        //                 );
        //             }
        //         }
    }
}
