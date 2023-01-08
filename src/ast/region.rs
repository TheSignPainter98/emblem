use crate::parser;
use core::fmt::{self, Display};
use parser::Location;

#[derive(Clone, Debug)]
pub struct Region<'i> {
    pub file_name: &'i str,
    pub src: &'i str,
    pub lines: (usize, usize),
    pub cols: (usize, usize),
    pub indices: (usize, usize),
}

impl<'i> Region<'i> {
    pub fn new(start: &Location<'i>, end: &Location<'i>) -> Self {
        Self {
            file_name: start.file_name,
            src: start.src,
            lines: (start.line, end.line),
            indices: (start.index, end.index),
            cols: (start.col, end.col),
        }
    }
}

impl Display for Region<'_> {
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
        let start = Location::new("fname.em", text);
        let end = start.clone().shift(text);

        let region = Region::new(&start, &end);

        assert_eq!("fname.em", region.file_name);
        assert_eq!(text, region.src);
        assert_eq!((start.line, end.line), region.lines);
        assert_eq!((start.col, end.col), region.cols);
        assert_eq!((start.index, end.index), region.indices);
    }
}
