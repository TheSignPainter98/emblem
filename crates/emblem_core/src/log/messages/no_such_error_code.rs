use crate::log::messages::Message;
use crate::log::{Log, LogId};
use derive_new::new;
use indoc::indoc;

#[derive(Default, new)]
pub struct NoSuchErrorCode {
    id: LogId,
}

impl Message for NoSuchErrorCode {
    fn id() -> LogId {
        "E001".into()
    }

    fn log(self) -> Log {
        Log::error(format!("no such error code ‘{}’", self.id))
            .with_id(Self::id())
            .explainable()
            .with_help("perhaps there is a typo here?")
    }

    fn explain(&self) -> &'static str {
        indoc! {"
            Error codes have the form `Eddd`, for digits `d`, such as this error, E001. If you're
            seeing this, please check for any typos.
        "}
    }
}
