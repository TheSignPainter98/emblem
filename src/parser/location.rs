use core::fmt::{self, Display};
use crate::parser::point::Point;

#[derive(Clone, Debug)]
pub struct Location<'i> {
    file_name: &'i str,
    src: &'i str,
    lines: (usize, usize),
    cols: (usize, usize),
    indices: (usize, usize),
}

impl<'i> Location<'i> {
    #[allow(dead_code)]
    pub fn new(start: &Point<'i>, end: &Point<'i>) -> Self {
        Self {
            file_name: start.file_name,
            src: start.src,
            lines: (start.line, end.line),
            indices: (start.index, end.index),
            cols: (start.col, end.col),
        }
    }

    pub fn file_name(&self) -> &str {
        self.file_name
    }

    pub fn src(&self) -> &str {
        self.src
    }

    pub fn lines(&self) -> (usize, usize) {
        self.lines
    }

    pub fn cols(&self) -> (usize, usize) {
        self.cols
    }

    pub fn indices(&self) -> (usize, usize) {
        self.indices
    }
}

impl Display for Location<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.lines.0 != self.lines.1 {
            write!(
                f,
                "{}:{}-{}:{}-{}",
                self.file_name, self.lines.0, self.lines.1, self.cols.0, self.cols.1
            )
        } else if self.cols.0 != self.cols.1 {
            write!(
                f,
                "{}:{}:{}-{}",
                self.file_name, self.lines.0, self.cols.0, self.cols.1
            )
        } else {
            write!(f, "{}:{}:{}", self.file_name, self.lines.0, self.cols.0)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let text = "my name\nis methos";
        let start = Point::new("fname.em", text);
        let end = start.clone().shift(text);

        let region = Location::new(&start, &end);

        assert_eq!("fname.em", region.file_name());
        assert_eq!(text, region.src());
        assert_eq!((start.line, end.line), region.lines());
        assert_eq!((start.col, end.col), region.cols());
        assert_eq!((start.index, end.index), region.indices());
    }
}
