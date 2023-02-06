use crate::log::messages::Message;
use crate::log::Log;
use derive_new::new;

#[derive(Default, new)]
pub struct NoSuchErrorCode<'a> {
    id: &'a str,
}

impl<'a> Message<'a> for NoSuchErrorCode<'a> {
    fn id() -> &'static str
    where
        Self: Sized,
    {
        "E001"
    }

    fn log(self) -> Log<'a> {
        Log::error(format!("no such error code {:?}", self.id))
            .id(Self::id())
            .explainable()
            .help("perhaps there is a typo here?")
    }

    fn explain(&self) -> &'static str {
        concat!(
            "Error codes have the form `Eddd`, for digits `d`, such as this error, E001. ",
            "If you're seeing this, please check for any typos."
        )
    }
}
