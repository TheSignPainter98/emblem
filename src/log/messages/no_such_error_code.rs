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
        "E000"
    }

    fn log(self) -> Log<'a> {
        Log::error(format!("no such error code {:?}", self.id))
            .id(Self::id())
            .help("perhaps this is a typo?")
    }

    fn explain() -> &'static str
    where
        Self: Sized,
    {
        concat!(
            "Error codes have the form `Eddd`, for digits `d`, such as this error, E001. ",
            "If you're seeing this, please check any typos."
        )
    }
}
