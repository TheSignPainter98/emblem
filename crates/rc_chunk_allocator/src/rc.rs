use crate::rc_chunk::RcChunk;
use std::{
    fmt::{self, Debug},
    ops::Deref,
};

pub struct Rc<T: Debug, const N: usize> {
    chunk: RcChunk<T, N>,
    index: usize,
}

impl<T: Debug, const N: usize> Rc<T, N> {
    pub(crate) fn new(chunk: RcChunk<T, N>, index: usize) -> Self {
        Self { chunk, index }
    }
}

impl<T: Debug, const N: usize> Deref for Rc<T, N> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.chunk[self.index]
    }
}

impl<T: Debug, const N: usize> Debug for Rc<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.deref())
    }
}
