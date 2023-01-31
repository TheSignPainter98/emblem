mod unexpected_eof;

pub use unexpected_eof::UnexpectedEOF;

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

#[allow(dead_code)]
fn messages<'i>() -> Vec<(&'static str, &'static str)> {
    macro_rules! messages {
        ($($msg:ident),*) => {
            {
                let mut ret = Vec::new();
                $(
                    let id = $msg::id();
                    if id != "" {
                        ret.push((id, $msg::explain()));
                    }
                )*
                ret
            }
        };
    }

    messages![UnexpectedEOF]
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn unique() {
        let ids: Vec<_> = messages().into_iter().map(|(id, _)| id).collect();

        let mut seen = HashSet::new();
        for id in ids {
            assert!(seen.insert(id));
        }
    }

    mod descriptions {
        use super::*;

        #[test]
        fn not_too_long() {
            for (id, desc) in messages() {
                let nchars = desc.chars().count();
                assert!(
                    nchars <= 1000,
                    "{} description is too long: contains {} chars",
                    id,
                    nchars
                );
            }
        }

        // #[test]
        // fn not_too_short() {
        //     for (id, desc) in messages() {
        //         let nchars = desc.chars().count();
        //         assert!(
        //             nchars >= 100,
        //             "{} description is too short: contains {} chars",
        //             id,
        //             nchars
        //         );
        //     }
        // }
    }
}
