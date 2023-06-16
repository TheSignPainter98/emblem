use crate::rc_chunk_allocator_metrics::RcChunkAllocatorMetrics;
use std::{
    cell::RefCell,
    fmt::Debug,
    mem::{self, MaybeUninit},
    ops::Index,
    rc::Rc as StdRc,
};

pub(crate) struct RcChunk<T: Debug, const N: usize> {
    inner: StdRc<RefCell<RcChunkImpl<T, N>>>,
}

impl<T: Debug, const N: usize> RcChunk<T, N> {
    pub(crate) fn new(parent_metrics: RcChunkAllocatorMetrics<T, N>) -> Self {
        Self {
            inner: StdRc::new(RefCell::new(RcChunkImpl::new(parent_metrics))),
        }
    }

    pub(crate) fn try_alloc(&self, t: T) -> Result<usize, T> {
        self.inner
            .try_borrow_mut()
            .expect("internal error: chunk being mutated")
            .try_alloc(t)
    }

    pub(crate) fn size() -> usize {
        mem::size_of::<StdRc<RefCell<RcChunkImpl<T, N>>>>() + mem::size_of::<RcChunkImpl<T, N>>()
    }
}

impl<T: Debug, const N: usize> Clone for RcChunk<T, N> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Debug, const N: usize> Index<usize> for RcChunk<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.inner.as_ptr() }.index(index)
    }
}

struct RcChunkImpl<T: Debug, const N: usize> {
    parent_metrics: RcChunkAllocatorMetrics<T, N>,
    len: usize,
    chunk: [MaybeUninit<T>; N],
}

impl<T: Debug, const N: usize> RcChunkImpl<T, N> {
    fn new(parent_metrics: RcChunkAllocatorMetrics<T, N>) -> Self {
        Self {
            parent_metrics,
            len: 0,
            chunk: unsafe { MaybeUninit::uninit().assume_init() },
        }
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

impl<T: Debug, const N: usize> Index<usize> for RcChunkImpl<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len);
        unsafe { self.chunk[index].assume_init_ref() }
    }
}

impl<T: Debug, const N: usize> Drop for RcChunkImpl<T, N> {
    fn drop(&mut self) {
        println!("dropping chunk");
        for elem in &mut self.chunk[..self.len] {
            unsafe {
                elem.assume_init_drop();
            }
        }

        self.parent_metrics.on_child_dropped();
    }
}
