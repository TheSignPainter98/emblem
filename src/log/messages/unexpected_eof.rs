use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::{Location, Point};

#[derive(Default)]
pub struct UnexpectedEOF<'i> {
    point: Point<'i>,
    expected: Vec<String>,
}

impl<'i> UnexpectedEOF<'i> {
    pub fn new(mut point: Point<'i>, expected: Vec<String>) -> Self {
        assert!(
            point.index > 0,
            "internal error: empty files are supposed to be valid"
        );

        point.index -= 1;

        Self { point, expected }
    }
}

impl<'i> Message<'i> for UnexpectedEOF<'i> {
    fn log(self) -> Log<'i> {
        let loc = Location::new(&self.point, &self.point.clone().shift("\0"));
        Log::error("unexpected eof")
            .id(Self::id())
            .src(Src::new(&loc).annotate(Note::error(&loc, "file ended early here")))
            .expect_one_of(&self.expected)
    }
}
