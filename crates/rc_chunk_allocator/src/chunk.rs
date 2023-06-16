use crate::RcChunkAllocator;
use std::{
    cell::RefCell,
    fmt::Debug,
    mem::{self, MaybeUninit},
    ops::Index,
    rc::Rc as StdRc,
};

pub(crate) struct Chunk<T: Debug, const N: usize> {
    inner: StdRc<RefCell<ChunkImpl<T, N>>>,
}

impl<T: Debug, const N: usize> Chunk<T, N> {
    pub(crate) fn new(parent: RcChunkAllocator<T, N>) -> Self {
        Self {
            inner: StdRc::new(RefCell::new(ChunkImpl::new(parent))),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.inner.try_borrow().unwrap().is_empty()
    }

    pub(crate) fn try_alloc(&self, t: T) -> Result<usize, T> {
        self.inner.try_borrow_mut().unwrap().try_alloc(t)
    }

    pub(crate) fn size() -> usize {
        mem::size_of::<StdRc<RefCell<ChunkImpl<T, N>>>>() + mem::size_of::<ChunkImpl<T, N>>()
    }
}

impl<T: Debug, const N: usize> Clone for Chunk<T, N> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Debug, const N: usize> Index<usize> for Chunk<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.inner.as_ptr() }.index(index)
    }
}

struct ChunkImpl<T: Debug, const N: usize> {
    parent: RcChunkAllocator<T, N>,
    len: usize,
    chunk: [MaybeUninit<T>; N],
}

impl<T: Debug, const N: usize> ChunkImpl<T, N> {
    fn new(parent: RcChunkAllocator<T, N>) -> Self {
        Self {
            parent,
            len: 0,
            chunk: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn try_alloc(&mut self, t: T) -> Result<usize, T> {
        if self.len >= N {
            return Err(t);
        }

        self.chunk[self.len].write(t);
        self.len += 1;

        Ok(self.len)
    }
}

impl<T: Debug, const N: usize> Index<usize> for ChunkImpl<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len);
        unsafe { self.chunk[index].assume_init_ref() }
    }
}

impl<T: Debug, const N: usize> Drop for ChunkImpl<T, N> {
    fn drop(&mut self) {
        println!("dropping chunk");
        for elem in &mut self.chunk[..self.len] {
            unsafe {
                elem.assume_init_drop();
            }
        }

        self.parent.on_child_dropped();
    }
}
