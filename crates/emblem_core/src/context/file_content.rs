use std::{
    borrow::Borrow,
    fmt::Display,
    ops::{Bound, Range, RangeBounds},
    rc::Rc,
};

pub trait FileSlice: AsRef<str> {
    fn slice<R: RangeBounds<usize>>(&self, index: R) -> FileContentSlice;
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

impl Borrow<str> for FileContent {
    fn borrow(&self) -> &str {
        self.inner.borrow()
    }
}

impl PartialEq<&str> for FileContent {
    fn eq(&self, other: &&str) -> bool {
        self.inner.as_ref() == *other
    }
}

impl PartialEq<FileContent> for &str {
    fn eq(&self, other: &FileContent) -> bool {
        *self == other.inner.as_ref()
    }
}

impl PartialEq<&str> for &FileContent {
    fn eq(&self, other: &&str) -> bool {
        self.inner.as_ref() == *other
    }
}

impl PartialEq<&FileContent> for &str {
    fn eq(&self, other: &&FileContent) -> bool {
        *self == other.inner.as_ref()
    }
}

impl Display for FileContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct FileContentSlice {
    raw: FileContent,
    range: Range<usize>,
}

impl FileSlice for FileContentSlice {
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

impl AsRef<str> for FileContentSlice {
    fn as_ref(&self) -> &str {
        &self.raw.as_ref()[self.range.clone()]
    }
}

impl Display for FileContentSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
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
}
