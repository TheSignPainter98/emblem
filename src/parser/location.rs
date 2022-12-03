use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref NEWLINE: Regex = Regex::new("(\n|\r\n|\r)").unwrap();
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Location<'input> {
    pub file_name: &'input str,
    pub src: &'input str,
    pub index: usize,
    pub line: usize,
}

impl<'input> Location<'input> {
    pub fn new(fname: &'input str, src: &'input str) -> Self {
        Self {
            file_name: fname,
            src,
            index: 0,
            line: 1,
        }
    }

    pub fn shift(mut self, text: &'input str) -> Self {
        self.index += text.len();
        self.line += NEWLINE.split(text).count() - 1;
        self
    }

    pub fn text_upto(&self, other: &Location) -> &'input str {
        &self.src[self.index..other.index]
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
    }

    #[test]
    fn shift_single_line() {
        let src = "my name is methos";
        let start = Location::new("fname", src);
        let mid = start.shift("my name is ");
        let end = mid.shift("methos");

        assert_eq!("fname", mid.file_name);
        assert_eq!(src, mid.src);
        assert_eq!(11, mid.index);
        assert_eq!(1, mid.line);

        assert_eq!("fname", end.file_name);
        assert_eq!(src, end.src);
        assert_eq!(17, end.index);
        assert_eq!(1, end.line);

        assert_eq!("my name is ", start.text_upto(&mid));
        assert_eq!("methos", mid.text_upto(&end));
        assert_eq!("my name is methos", start.text_upto(&end));
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
        assert_eq!(src, start.text_upto(&end));
    }
}
