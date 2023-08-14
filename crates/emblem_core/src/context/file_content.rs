use std::{
    fmt::Display,
    ops::{Bound, Deref, Range, RangeBounds},
    rc::Rc,
};

#[cfg(test)]
use crate::ast::AstDebug;

pub trait FileSlice: AsRef<str> {
    fn raw(&self) -> &str;
    fn slice<R: RangeBounds<usize>>(&self, index: R) -> FileContentSlice;

    fn to_str(&self) -> &str {
        self.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileContent {
    inner: Rc<str>,
}

impl FileContent {
    pub(crate) fn new(contents: &str) -> Self {
        Self {
            inner: contents.into(),
        }
    }
}

impl Default for FileContent {
    fn default() -> Self {
        Self::new("")
    }
}

impl FileSlice for FileContent {
    fn raw(&self) -> &str {
        self.as_ref()
    }

    fn slice<R: RangeBounds<usize>>(&self, index: R) -> FileContentSlice {
        let start = match index.start_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(_) => panic!("internal error: excluded left lower bound"),
            Bound::Unbounded => 0,
        };
        let end = match index.end_bound() {
            Bound::Included(i) => *i + 1,
            Bound::Excluded(i) => *i,
            Bound::Unbounded => self.as_ref().len(),
        };
        FileContentSlice {
            raw: self.clone(),
            range: start..end,
        }
    }
}

impl AsRef<str> for FileContent {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}

impl Deref for FileContent {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.to_str()
    }
}

impl PartialEq<&str> for FileContent {
    fn eq(&self, rhs: &&str) -> bool {
        self.to_str() == *rhs
    }
}

impl PartialEq<FileContent> for &str {
    fn eq(&self, rhs: &FileContent) -> bool {
        *self == rhs.to_str()
    }
}
//
// impl Borrow<str> for FileContent {
//     fn borrow(&self) -> &str {
//         self.inner.borrow()
//     }
// }

// impl PartialEq<FileContent> for &str {
//     fn eq(&self, other: &FileContent) -> bool {
//         *self == other.to_str()
//     }
// }
//
impl Display for FileContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_str().fmt(f)
    }
}

#[derive(Clone, Debug, Eq)]
pub struct FileContentSlice {
    raw: FileContent,
    range: Range<usize>,
}

impl FileContentSlice {
    pub(crate) fn range(&self) -> &Range<usize> {
        &self.range
    }

    pub(crate) fn trim(self) -> Self {
        let start = self.chars().position(is_non_whitespace).unwrap_or_default();
        let len = self.len();
        let end = len
            - self
                .chars()
                .rev()
                .position(is_non_whitespace)
                .unwrap_or_default();
        if (start..end) != (0..len) {
            self.slice(start..end)
        } else {
            self
        }
    }

    pub(crate) fn trimmed(&self) -> Self {
        self.clone().trim()
    }

    #[allow(dead_code)]
    pub(crate) fn trimmed_left(&self) -> Self {
        let start = self.chars().position(is_non_whitespace).unwrap_or_default();
        if start != 0 {
            self.slice(start..)
        } else {
            self.clone()
        }
    }

    pub(crate) fn trimmed_right(&self) -> Self {
        let len = self.len();
        let end = len
            - self
                .chars()
                .rev()
                .position(is_non_whitespace)
                .unwrap_or_default();
        if end != len {
            self.slice(..end)
        } else {
            self.clone()
        }
    }
}

fn is_non_whitespace(c: char) -> bool {
    c != ' ' && c != '\t'
}

impl Default for FileContentSlice {
    fn default() -> Self {
        let raw = FileContent::default();
        let range = 0..raw.len();
        Self { raw, range }
    }
}

impl FileSlice for FileContentSlice {
    fn raw(&self) -> &str {
        self.raw.to_str()
    }

    fn slice<R: RangeBounds<usize>>(&self, index: R) -> FileContentSlice {
        let start = self.range.start
            + match index.start_bound() {
                Bound::Included(i) => *i,
                Bound::Excluded(_) => panic!("internal error: excluded left lower bound"),
                Bound::Unbounded => 0,
            };
        let end = self.range.end
            + match index.end_bound() {
                Bound::Included(i) => *i + 1,
                Bound::Excluded(i) => *i,
                Bound::Unbounded => self.as_ref().len(),
            };
        Self {
            raw: self.raw.clone(),
            range: start..end,
        }
    }
}
impl From<FileContent> for FileContentSlice {
    fn from(content: FileContent) -> Self {
        content.slice(..)
    }
}

impl AsRef<str> for FileContentSlice {
    fn as_ref(&self) -> &str {
        &self.raw.as_ref()[self.range.clone()]
    }
}

impl Deref for FileContentSlice {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.to_str()
    }
}

impl PartialEq for FileContentSlice {
    fn eq(&self, rhs: &Self) -> bool {
        self.to_str() == rhs.to_str()
    }
}

impl PartialEq<String> for FileContentSlice {
    fn eq(&self, rhs: &String) -> bool {
        self.to_str() == rhs
    }
}

impl PartialEq<FileContentSlice> for String {
    fn eq(&self, rhs: &FileContentSlice) -> bool {
        *self == rhs.to_str()
    }
}

impl PartialEq<str> for FileContentSlice {
    fn eq(&self, rhs: &str) -> bool {
        self.to_str() == rhs
    }
}

impl PartialEq<FileContentSlice> for &str {
    fn eq(&self, rhs: &FileContentSlice) -> bool {
        *self == rhs.to_str()
    }
}

impl Display for FileContentSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

#[cfg(test)]
impl AstDebug for FileContentSlice {
    fn test_fmt(&self, buf: &mut Vec<String>) {
        self.to_str().test_fmt(buf)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn slice_unbounded() {
        let original = "hello big world";
        let content = FileContent::new(original);
        assert_eq!(original, content.slice(..).as_ref());
    }

    #[test]
    fn slice_exclusive() {
        let original = "hello big world";
        let content = FileContent::new(original);
        let range = 1..10;
        assert_eq!(&original[range.clone()], content.slice(range).as_ref());
    }

    #[test]
    fn slice_inclusive() {
        let original = "hello big world";
        let content = FileContent::new(original);
        let range = 1..=10;
        assert_eq!(&original[range.clone()], content.slice(range).as_ref());
    }

    #[test]
    fn trim() {
        assert_eq!(
            "we are flying high",
            FileContent::new("we are flying high").slice(..).trim()
        );
        assert_eq!(
            "saying farewell would be a lie",
            FileContent::new("   saying farewell would be a lie       ")
                .slice(..)
                .trim()
        );
        assert_eq!(
            "there's no need tonight",
            FileContent::new("\t\t\tthere's no need tonight\t")
                .slice(..)
                .trim()
        );
        assert_eq!(
            "to spend a sleepless, lonely night",
            FileContent::new("\t  \tthere's no need tonight\t\t    ")
                .slice(..)
                .trim()
        );
    }

    #[test]
    fn trimmed() {
        assert_eq!(
            "we are flying high",
            FileContent::new("we are flying high").slice(..).trimmed()
        );
        assert_eq!(
            "saying farewell would be a lie",
            FileContent::new("   saying farewell would be a lie       ")
                .slice(..)
                .trimmed()
        );
        assert_eq!(
            "there's no need tonight",
            FileContent::new("\t\t\tthere's no need tonight\t")
                .slice(..)
                .trimmed()
        );
        assert_eq!(
            "to spend a sleepless, lonely night",
            FileContent::new("\t  \tthere's no need tonight\t\t    ")
                .slice(..)
                .trimmed()
        );
    }

    #[test]
    fn trimmed_left() {
        assert_eq!(
            "we are flying high",
            FileContent::new("we are flying high")
                .slice(..)
                .trimmed_left()
        );
        assert_eq!(
            "there's no way to say goodbye       ",
            FileContent::new("   there's no way to say goodbye       ")
                .slice(..)
                .trimmed_left()
        );
        assert_eq!(
            "you're denying, why?\t",
            FileContent::new("\t\t\tyou're denying, why?\t")
                .slice(..)
                .trimmed_left()
        );
        assert_eq!(
            "I'll be back and then you'll be mine\t\t    ",
            FileContent::new("\t  \tI'll be back and then you'll be mine\t\t    ")
                .slice(..)
                .trimmed_left()
        );
    }

    #[test]
    fn trimmed_right() {
        assert_eq!(
            "It's night, you gotta be the fly",
            FileContent::new("It's night, you gotta be the fly")
                .slice(..)
                .trimmed_right(),
        );
        assert_eq!(
            "   Flying to the lamp, burning her little chances in the light",
            FileContent::new(
                "   Flying to the lamp, burning her little chances in the light       "
            )
            .slice(..)
            .trimmed_right()
        );
        assert_eq!(
            "\t\t\tShe'll never learn to bite, but for now you cannot fight",
            FileContent::new("\t\t\tShe'll never learn to bite, but for now you cannot fight\t")
                .slice(..)
                .trimmed_right()
        );
        assert_eq!(
            "And you're here to do it tight, burn your wings now and cry\t\t    ",
            FileContent::new(
                "\t  \tAnd you're here to do it tight, burn your wings now and cry\t\t    "
            )
            .slice(..)
            .trimmed_right()
        );
    }
}
