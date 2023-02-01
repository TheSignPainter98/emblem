use crate::log::messages::Message;
use crate::log::{Log, Msg, Src};
use crate::parser::{Location, Point};

#[derive(Default)]
pub struct UnexpectedEOF<'i> {
    point: Point<'i>,
    expected: Vec<String>,
}

impl<'i> UnexpectedEOF<'i> {
    pub fn new(point: Point<'i>, expected: Vec<String>) -> Self {
        Self { point, expected }
    }
}

impl<'i> Message<'i> for UnexpectedEOF<'i> {
    fn id() -> &'static str {
        "E001"
    }

    fn log(self) -> Log<'i> {
        let loc = Location::new(&self.point, &self.point.clone().shift("\0"));
        Log::error("unexpected eof")
            .id(Self::id())
            .src(Src::new(&loc).annotate(Msg::error(&loc, "file ended early here")))
            .expect_one_of(&self.expected)
    }
}
