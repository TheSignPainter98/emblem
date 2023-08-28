use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::{self, Display};

use crate::{FileContent, FileContentSlice, FileName};

lazy_static! {
    static ref NEWLINE: Regex = Regex::new("(\n|\r\n|\r)").unwrap();
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Point {
    // TODO(kcza): make methods for these
    pub file_name: FileName,
    pub src: FileContentSlice,
    pub line: usize,
    pub col: usize,
    pub index: usize,
}

impl Point {
    pub fn new(file_name: FileName, src: FileContent) -> Self {
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
}

impl Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[cfg(test)]
mod test {
    use crate::{context::file_content::FileSlice, Context};

    use super::*;
    #[test]
    fn new() {
        let ctx = Context::new();
        let src = "content";
        let loc = Point::new(ctx.alloc_file_name("fname"), ctx.alloc_file_content(src));

        assert_eq!("fname", &loc.file_name);
        assert_eq!(src, loc.src.to_str());
        assert_eq!(0, loc.index);
        assert_eq!(1, loc.line);
        assert_eq!(1, loc.col);
    }

    #[test]
    fn shift_single_line() {
        let ctx = Context::new();
        let src = "my name is methos";
        let start = Point::new(ctx.alloc_file_name("fname"), ctx.alloc_file_content(src));
        let mid = start.shift("my name is ");
        let end = mid.clone().shift("methos");

        assert_eq!("fname", mid.file_name);
        assert_eq!(src, mid.src);
        assert_eq!(11, mid.index);
        assert_eq!(1, mid.line);
        assert_eq!(12, mid.col);

        assert_eq!("fname", end.file_name);
        assert_eq!(src, end.src);
        assert_eq!(17, end.index);
        assert_eq!(1, end.line);
        assert_eq!(18, end.col);
    }

    #[test]
    fn tabs() {
        let ctx = Context::new();
        let src = "\thello,\tworld";
        let start = Point::new(ctx.alloc_file_name("fname"), ctx.alloc_file_content(src));
        let end = start.shift(src);

        assert_eq!(13, end.index);
        assert_eq!(20, end.col);
    }

    #[test]
    fn shift_multi_line() {
        let ctx = Context::new();
        let raw_src = "Welcome! Welcome to City 17! You have chosen, or been chosen, to relocate to one of our finest remaining urban centres";
        let src = raw_src.replace(' ', "\n");
        let start = Point::new(
            ctx.alloc_file_name("file_name"),
            ctx.alloc_file_content(&src),
        );
        let end = start.shift(&src);

        assert_eq!("file_name", end.file_name);
        assert_eq!(src, end.src);
        assert_eq!(21, end.line);
        assert_eq!(118, end.index);
        assert_eq!(8, end.col);
    }
}
