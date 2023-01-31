use crate::log::messages::Message;
use crate::log::{Log, Src, Msg};
use crate::parser::Location;

pub struct UnexpectedEOF<'i> {
    loc: Location<'i>,
}

impl<'i> Message<'i> for UnexpectedEOF<'i> {
    fn id() -> &'static str {
        "E001"
    }

    fn log(self) -> Log<'i> {
        Log::error("unexpected eof")
            .src(Src::new(&self.loc).annotate(Msg::error(&self.loc, "file ended early here")))
    }
}
