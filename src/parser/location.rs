use crate::parser::point::Point;
use core::fmt::{self, Display};
use std::cmp;

#[derive(Clone, Debug, Default)]
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
            cols: (start.col, cmp::max(1, end.col - 1)),
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

    pub fn span_to(&self, other: &Self) -> Self {
        if self.file_name != other.file_name {
            panic!(
                "internal error: attempted to span across files: {} and {}",
                self.file_name, other.file_name
            );
        }

        Self {
            file_name: other.file_name,
            src: self.src,
            lines: (
                cmp::min(self.lines.0, other.lines.0),
                cmp::max(self.lines.1, other.lines.1),
            ),
            indices: (
                cmp::min(self.indices.0, other.indices.0),
                cmp::max(self.indices.1, other.indices.1),
            ),
            cols: (
                cmp::min(self.cols.0, other.cols.0),
                cmp::max(self.cols.1, other.cols.1),
            ),
        }
    }
}

impl Display for Location<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.lines.0 != self.lines.1 {
            write!(
                f,
                "{}:{}:{}-{}:{}",
                self.file_name, self.lines.0, self.cols.0, self.lines.1, self.cols.1
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
    fn mid_line() {
        let text = "my name\nis methos";
        let start = Point::new("fname.em", text);
        let end = start.clone().shift(text);

        let loc = Location::new(&start, &end);

        assert_eq!("fname.em", loc.file_name());
        assert_eq!(text, loc.src());
        assert_eq!((start.line, end.line), loc.lines());
        assert_eq!((start.col, end.col - 1), loc.cols());
        assert_eq!((start.index, end.index), loc.indices());
    }

    #[test]
    fn end_of_line() {
        let text = "my name is methos\n";
        let start = Point::new("fname.em", text);
        let end = start.clone().shift(text);

        let loc = Location::new(&start, &end);

        assert_eq!("fname.em", loc.file_name());
        assert_eq!(text, loc.src());
        assert_eq!((start.line, end.line), loc.lines());
        assert_eq!((start.col, 1), loc.cols());
        assert_eq!((start.index, end.index), loc.indices());
    }

    #[test]
    fn span_to() {
        let text = "my name is methos\n";
        let p1 = Point::new("fname.em", text);
        let p2 = p1.clone().shift("my name");
        let p3 = p2.clone().shift(" is ");
        let p4 = p2.clone().shift("methos");

        for (l1, l2) in vec![
            (Location::new(&p1, &p2), Location::new(&p3, &p4)),
            (Location::new(&p1, &p3), Location::new(&p2, &p4)),
            (Location::new(&p1, &p4), Location::new(&p2, &p3)),
        ] {
            for (l1, l2) in vec![(&l1, &l2), (&l2, &l1)] {
                let span = l1.span_to(l2);
                assert_eq!(
                    span.indices.0,
                    cmp::min(
                        cmp::min(l1.indices.0, l1.indices.1),
                        cmp::min(l2.indices.0, l2.indices.1)
                    )
                );
                assert_eq!(
                    span.indices.1,
                    cmp::max(
                        cmp::max(l1.indices.0, l1.indices.1),
                        cmp::max(l2.indices.0, l2.indices.1)
                    )
                );
            }
        }
    }
}
