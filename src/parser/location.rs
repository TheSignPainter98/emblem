use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::{self, Display};

lazy_static! {
    static ref NEWLINE: Regex = Regex::new("(\n|\r\n|\r)").unwrap();
}

#[derive(Clone, Debug, Default)]
pub struct Location<'input> {
    pub file_name: &'input str,
    pub src: &'input str,
    pub line: usize,
    pub col: usize,
    pub index: usize,
}

impl<'input> Location<'input> {
    pub fn new(fname: &'input str, src: &'input str) -> Self {
        Self {
            file_name: fname,
            src,
            index: 0,
            line: 1,
            col: 0,
        }
    }

    pub fn shift(mut self, text: &'input str) -> Self {
        let lines: Vec<&str> = NEWLINE.split(text).into_iter().collect();
        let num_lines = lines.len();

        self.line += num_lines - 1;

        let last_line = lines[num_lines - 1];
        let last_line_width = last_line
            .chars()
            .map(|c| if c == '\t' { 4 } else { 1 })
            .sum();
        self.col = if num_lines > 1 {
            last_line_width
        } else {
            self.col + last_line_width
        };

        self.index += text.len();

        self
    }

    #[allow(dead_code)]
    pub fn text_upto(&self, other: &Location) -> &'input str {
        &self.src[self.index..other.index]
    }
}

impl<'input> Display for Location<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:({})", self.file_name, self.line, self.index)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new() {
        let src = "content";
        let loc = Location::new("fname", src);

        assert_eq!("fname", loc.file_name);
        assert_eq!(src, loc.src);
        assert_eq!(0, loc.index);
        assert_eq!(1, loc.line);
        assert_eq!(0, loc.col);
    }

    #[test]
    fn shift_single_line() {
        let src = "my name is methos";
        let start = Location::new("fname", src);
        let mid = start.clone().shift("my name is ");
        let end = mid.clone().shift("methos");

        assert_eq!("fname", mid.file_name);
        assert_eq!(src, mid.src);
        assert_eq!(11, mid.index);
        assert_eq!(1, mid.line);
        assert_eq!(11, mid.col);

        assert_eq!("fname", end.file_name);
        assert_eq!(src, end.src);
        assert_eq!(17, end.index);
        assert_eq!(1, end.line);
        assert_eq!(17, end.col);

        assert_eq!("my name is ", start.text_upto(&mid));
        assert_eq!("methos", mid.text_upto(&end));
        assert_eq!("my name is methos", start.text_upto(&end));
    }

    #[test]
    fn tabs() {
        let src = "\thello,\tworld";
        let start = Location::new("fname", src);
        let end = start.shift(src);

        assert_eq!(13, end.index);
        assert_eq!(19, end.col);
    }

    #[test]
    fn shift_multi_line() {
        let raw_src = "Welcome! Welcome to City 17! You have chosen, or been chosen, to relocate to one of our finest remaining urban centres";
        let src = raw_src.replace(" ", "\n");
        let start = Location::new("file_name", &src);
        let end = start.clone().shift(&src);

        assert_eq!("file_name", end.file_name);
        assert_eq!(src, end.src);
        assert_eq!(21, end.line);
        assert_eq!(118, end.index);
        assert_eq!(7, end.col);

        assert_eq!(src, start.text_upto(&end));
    }
}
