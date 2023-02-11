use crate::parser::{LocationContext, Point};
use core::fmt::{self, Display};
use std::cmp;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Location<'i> {
    file_name: &'i str,
    src: &'i str,
    lines: (usize, usize),
    cols: (usize, usize),
    indices: (usize, usize),
}

impl<'i> Location<'i> {
    pub fn new(start: &Point<'i>, end: &Point<'i>) -> Self {
        Self {
            file_name: start.file_name,
            src: start.src,
            lines: (start.line, end.line),
            indices: (start.index, end.index),
            cols: (start.col, cmp::max(1, end.col - 1)),
        }
    }

    pub fn file_name(&self) -> &'i str {
        self.file_name
    }

    pub fn src(&self) -> &'i str {
        self.src
    }

    pub fn lines(&self) -> (usize, usize) {
        self.lines
    }

    pub fn cols(&self) -> (usize, usize) {
        self.cols
    }

    pub fn indices(&self, context: &LocationContext) -> (usize, usize) {
        let (start, end) = self.indices;
        let context_start = context.starting_index();
        (start - context_start, end - context_start)
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

    pub fn context(&self) -> LocationContext<'i> {
        let start = self.src[..self.indices.0]
            .rfind(['\r', '\n'])
            .map(|i| i + 1)
            .unwrap_or_default();
        let end = self.src[self.indices.1..]
            .find(['\r', '\n'])
            .map(|i| i + self.indices.1)
            .unwrap_or(self.src.len());

        LocationContext::new(&self.src[start..end], start)
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

    mod lines_cols {
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
        }
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
            for (l1, l2) in &[(&l1, &l2), (&l2, &l1)] {
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

    mod context {
        use super::*;
        #[test]
        fn single_line() {
            let text = "oh! santiana gained a day";
            let text_start = Point::new("fname.em", text);

            let loc_start_shift = "oh! ";
            let loc_text = "santiana";

            let loc_start = text_start.clone().shift(loc_start_shift);
            let loc_end = loc_start.clone().shift(loc_text);

            let loc = Location::new(&loc_start, &loc_end);
            let context = loc.context();
            assert_eq!(context.src(), text);
            assert_eq!(context.starting_index(), 0);
            assert_eq!(loc.indices(&context), (4, 12));
        }

        #[test]
        fn multi_line() {
            let lines = [
                "oh! santiana gained a day",
                "away santiana!",
                "'napoleon of the west,' they say",
                "along the plains of mexico",
            ];
            for newline in ["\n", "\r", "\r\n"] {
                let text = lines.join(newline);
                let text_start = Point::new("fname.em", &text);

                let loc_start_shift = &format!("oh! santiana gained a day{newline}away ");
                let loc_text = &format!("santiana!{newline}'napoleon of");

                let loc_start = text_start.clone().shift(loc_start_shift);
                let loc_end = loc_start.clone().shift(loc_text);

                let loc = Location::new(&loc_start, &loc_end);
                let context = loc.context();

                assert_eq!(context.src(), lines[1..3].join(newline));
                assert_eq!(context.starting_index(), 25 + newline.len());
                assert_eq!(loc.indices(&context), (5, 26 + newline.len()));
            }
        }
    }
}
