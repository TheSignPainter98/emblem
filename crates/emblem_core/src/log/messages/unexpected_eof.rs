use crate::log::messages::Message;
use crate::log::{Log, Note, Src};
use crate::parser::{Location, Point};

#[derive(Default)]
pub struct UnexpectedEOF {
    point: Point,
    expected: Vec<String>,
}

impl UnexpectedEOF {
    pub fn new(mut point: Point, expected: Vec<String>) -> Self {
        assert!(
            point.index > 0,
            "internal error: empty files are supposed to be valid"
        );

        point.index -= 1;

        Self { point, expected }
    }
}

impl<'i> Message<'i> for UnexpectedEOF {
    fn log(self) -> Log {
        let loc = Location::new(&self.point, &self.point.clone().shift("\0"));
        Log::error("unexpected eof")
            .with_src(Src::new(&loc).with_annotation(Note::error(&loc, "file ended early here")))
            .with_expected(self.expected)
    }
}
