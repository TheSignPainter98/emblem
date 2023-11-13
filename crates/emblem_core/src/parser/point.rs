use derive_new::new;
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::{self, Display};

use crate::{FileContent, FileContentSlice, FileName};

lazy_static! {
    static ref NEWLINE: Regex = Regex::new("(\n|\r\n|\r)").unwrap();
}

#[derive(Clone, Debug, Default, PartialEq, Eq, new)]
pub struct Point {
    file_name: FileName,
    src: FileContentSlice,
    line: usize,
    col: usize,
    index: usize,
}

impl Point {
    pub fn at_start_of(file_name: FileName, src: FileContent) -> Self {
        Self {
            file_name,
            src: src.into(),
            index: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn shift(mut self, text: &str) -> Self {
        let lines: Vec<&str> = NEWLINE.split(text).collect();
        let num_lines = lines.len();

        self.line += num_lines - 1;

        let last_line = lines[num_lines - 1];
        let last_line_width: usize = last_line
            .chars()
            .map(|c| if c == '\t' { 4 } else { 1 })
            .sum();
        self.col = last_line_width + if num_lines > 1 { 1 } else { self.col };

        self.index += text.len();

        self
    }

    pub fn file_name(&self) -> &FileName {
        &self.file_name
    }

    pub fn src(&self) -> &FileContentSlice {
        &self.src
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn index_mut(&mut self) -> &mut usize {
        &mut self.index
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        self.col
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[cfg(test)]
mod test {
    use crate::Context;

    use super::*;
    #[test]
    fn new() {
        let ctx = Context::test_new();
        let src = "content";
        let loc = Point::at_start_of(ctx.alloc_file_name("fname"), ctx.alloc_file_content(src));

        assert_eq!("fname", &loc.file_name);
        assert_eq!(src, loc.src());
        assert_eq!(0, loc.index());
        assert_eq!(1, loc.line());
        assert_eq!(1, loc.col());
    }

    #[test]
    fn shift_single_line() {
        let ctx = Context::test_new();
        let src = "my name is methos";
        let start = Point::at_start_of(ctx.alloc_file_name("fname"), ctx.alloc_file_content(src));
        let mid = start.shift("my name is ");
        let end = mid.clone().shift("methos");

        assert_eq!("fname", mid.file_name);
        assert_eq!(src, mid.src());
        assert_eq!(11, mid.index());
        assert_eq!(1, mid.line());
        assert_eq!(12, mid.col());

        assert_eq!("fname", end.file_name());
        assert_eq!(src, end.src());
        assert_eq!(17, end.index());
        assert_eq!(1, end.line());
        assert_eq!(18, end.col());
    }

    #[test]
    fn tabs() {
        let ctx = Context::test_new();
        let src = "\thello,\tworld";
        let start = Point::at_start_of(ctx.alloc_file_name("fname"), ctx.alloc_file_content(src));
        let end = start.shift(src);

        assert_eq!(13, end.index());
        assert_eq!(20, end.col());
    }

    #[test]
    fn shift_multi_line() {
        let ctx = Context::test_new();
        let raw_src = "Welcome! Welcome to City 17! You have chosen, or been chosen, to relocate to one of our finest remaining urban centres";
        let src = raw_src.replace(' ', "\n");
        let start = Point::at_start_of(
            ctx.alloc_file_name("file_name"),
            ctx.alloc_file_content(&src),
        );
        let end = start.shift(&src);

        assert_eq!("file_name", end.file_name());
        assert_eq!(src, end.src());
        assert_eq!(21, end.line());
        assert_eq!(118, end.index());
        assert_eq!(8, end.col());
    }
}
