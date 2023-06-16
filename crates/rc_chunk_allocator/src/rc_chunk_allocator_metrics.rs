use crate::rc_chunk::RcChunk;
use std::{cell::RefCell, fmt::Debug, marker::PhantomData, rc::Rc as StdRc};

pub struct RcChunkAllocatorMetrics<T: Debug, const N: usize> {
    inner: StdRc<RefCell<RcChunkAllocatorMetricsImpl<T, N>>>,
}

impl<T: Debug, const N: usize> RcChunkAllocatorMetrics<T, N> {
    pub(crate) fn new() -> Self {
        Self {
            inner: StdRc::new(RefCell::new(RcChunkAllocatorMetricsImpl::new())),
        }
    }

    pub fn memory_used(&self) -> usize {
        self.inner.try_borrow().unwrap().memory_used()
    }

    pub fn on_child_created(&self) {
        self.inner.try_borrow_mut().unwrap().on_child_created()
    }

    pub fn on_child_dropped(&self) {
        self.inner.try_borrow_mut().unwrap().on_child_dropped()
    }
}

impl<T: Debug, const N: usize> Clone for RcChunkAllocatorMetrics<T, N> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct RcChunkAllocatorMetricsImpl<T: Debug, const N: usize> {
    children: usize,
    phantom: PhantomData<T>,
}

impl<T: Debug, const N: usize> RcChunkAllocatorMetricsImpl<T, N> {
    fn new() -> Self {
        Self {
            children: 0,
            phantom: PhantomData,
        }
    }

    fn memory_used(&self) -> usize {
        self.children * RcChunk::<T, N>::size()
    }

    fn on_child_created(&mut self) {
        self.children += 1;
    }

    fn on_child_dropped(&mut self) {
        self.children -= 1;
    }
}
