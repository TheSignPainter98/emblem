use crate::{context::file_content::FileSlice, parser::Point, FileContentSlice, FileName};
use core::fmt::{self, Display};
use std::cmp;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Location {
    file_name: FileName,
    src: FileContentSlice,
    lines: (usize, usize),
    cols: (usize, usize),
    indices: (usize, usize),
}

impl Location {
    pub fn new(start: &Point, end: &Point) -> Self {
        Self {
            file_name: start.file_name.clone(),
            src: start.src.clone(),
            lines: (start.line, end.line),
            indices: (start.index, end.index),
            cols: (start.col, cmp::max(1, end.col - 1)),
        }
    }

    pub fn file_name(&self) -> &FileName {
        &self.file_name
    }

    pub fn src(&self) -> &FileContentSlice {
        &self.src
    }

    pub fn lines(&self) -> (usize, usize) {
        self.lines
    }

    pub fn cols(&self) -> (usize, usize) {
        self.cols
    }

    pub fn indices_in(&self, context: &FileContentSlice) -> (usize, usize) {
        let (start, end) = self.indices;
        let context_start = context.range().start;
        (start - context_start, end - context_start)
    }

    pub fn start(&self) -> Point {
        Point {
            file_name: self.file_name.clone(),
            src: self.src.clone(),
            line: self.lines.0,
            col: self.cols.0,
            index: self.indices.0,
        }
    }

    pub fn end(&self) -> Point {
        Point {
            file_name: self.file_name.clone(),
            src: self.src.clone(),
            line: self.lines.1,
            col: self.cols.1,
            index: self.indices.1,
        }
    }

    pub fn span_to(&self, other: &Self) -> Self {
        if self.file_name != other.file_name {
            panic!(
                "internal error: attempted to span across files: {} and {}",
                self.file_name, other.file_name
            );
        }

        Self {
            file_name: other.file_name.clone(),
            src: self.src.clone(),
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

    pub fn context(&self) -> FileContentSlice {
        let raw = self.src.raw();
        let start = raw[..self.indices.0]
            .rfind(['\r', '\n'])
            .map(|i| i + 1)
            .unwrap_or_default();
        let end = raw[self.indices.1..]
            .find(['\r', '\n'])
            .map(|i| i + self.indices.1)
            .unwrap_or(raw.len());
        self.src.slice(start..end)
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.lines.0 != self.lines.1 {
            write!(
                f,
                "{}:{}:{}-{}:{}",
                self.file_name, self.lines.0, self.cols.0, self.lines.1, self.cols.1
            )
        } else {
            write!(
                f,
                "{}:{}:{}-{}",
                self.file_name, self.lines.0, self.cols.0, self.cols.1
            )
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Context;

    use super::*;

    mod lines_cols {
        use crate::Context;

        use super::*;
        #[test]
        fn mid_line() {
            let ctx = Context::new();
            let text = "my name\nis methos";
            let start = Point::new(
                ctx.alloc_file_name("fname.em"),
                ctx.alloc_file_content(text),
            );
            let end = start.clone().shift(text);
            let loc = Location::new(&start, &end);
            assert_eq!("fname.em", loc.file_name());
            assert_eq!(text, loc.src().raw());
            assert_eq!((start.line, end.line), loc.lines());
            assert_eq!((start.col, end.col - 1), loc.cols());
        }

        #[test]
        fn end_of_line() {
            let ctx = Context::new();
            let text = "my name is methos\n";
            let start = Point::new(
                ctx.alloc_file_name("fname.em"),
                ctx.alloc_file_content(text),
            );
            let end = start.clone().shift(text);
            let loc = Location::new(&start, &end);
            assert_eq!("fname.em", loc.file_name());
            assert_eq!(text, loc.src().raw());
            assert_eq!((start.line, end.line), loc.lines());
            assert_eq!((start.col, 1), loc.cols());
        }
    }

    #[test]
    fn start() {
        let ctx = Context::new();
        let text = "my name is methos\n";
        let start = Point::new(
            ctx.alloc_file_name("fname.em"),
            ctx.alloc_file_content(text),
        );
        let end = start.clone().shift(text);
        let loc = Location::new(&start, &end);
        assert_eq!(loc.start(), start);
    }

    #[test]
    fn end() {
        let ctx = Context::new();
        let text = "my name is methos\n";
        let start = Point::new(
            ctx.alloc_file_name("fname.em"),
            ctx.alloc_file_content(text),
        );
        let end = start.clone().shift(text);
        let loc = Location::new(&start, &end);
        assert_eq!(loc.end(), end);
    }

    #[test]
    fn span_to() {
        let ctx = Context::new();
        let text = "my name is methos\n";
        let p1 = Point::new(
            ctx.alloc_file_name("fname.em"),
            ctx.alloc_file_content(text),
        );
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
            let ctx = Context::new();
            let text = "oh! santiana gained a day";
            let text_start = Point::new(
                ctx.alloc_file_name("fname.em"),
                ctx.alloc_file_content(text),
            );

            let loc_start_shift = "oh! ";
            let loc_text = "santiana";

            let loc_start = text_start.clone().shift(loc_start_shift);
            let loc_end = loc_start.clone().shift(loc_text);

            let loc = Location::new(&loc_start, &loc_end);
            let context = loc.context();
            assert_eq!(context.raw(), text);
            assert_eq!(*context.range(), 0..text.len());
            assert_eq!(loc.indices_in(&context), (4, 12));
        }

        #[test]
        fn multi_line() {
            let ctx = Context::new();
            let lines = [
                "oh! santiana gained a day",
                "away santiana!",
                "'napoleon of the west,' they say",
                "along the plains of mexico",
            ];
            for newline in ["\n", "\r", "\r\n"] {
                let text = lines.join(newline);
                let text_start = Point::new(
                    ctx.alloc_file_name("fname.em"),
                    ctx.alloc_file_content(&text),
                );

                let loc_start_shift = &format!("oh! santiana gained a day{newline}away ");
                let loc_text = &format!("santiana!{newline}'napoleon of");

                let loc_start = text_start.clone().shift(loc_start_shift);
                let loc_end = loc_start.clone().shift(loc_text);

                let loc = Location::new(&loc_start, &loc_end);
                let context = loc.context();

                assert_eq!(context.raw(), lines[1..3].join(newline));
                assert_eq!(*context.range(), 25 + newline.len()..lines.len());
                assert_eq!(loc.indices_in(&context), (5, 26 + newline.len()));
            }
        }
    }
}
