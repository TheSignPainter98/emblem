use crate::rc_chunk::RcChunk;
use std::{fmt::Debug, marker::PhantomData, rc::Rc as StdRc, cell::RefCell};

pub(crate) struct RcChunkAllocatorMetrics<T: Debug, const N: usize> {
    inner: StdRc<RefCell<RcChunkAllocatorMetricsImpl<T, N>>>,
}

impl<T: Debug, const N: usize> RcChunkAllocatorMetrics<T, N> {
    pub(crate) fn new() -> Self {
        Self {
            inner: StdRc::new(RefCell::new(RcChunkAllocatorMetricsImpl::new())),
        }
    }

    pub(crate) fn memory_used(&self) -> usize {
        self.inner.try_borrow_mut().unwrap().memory_used()
    }

    pub(crate) fn on_child_created(&self) {
        self.inner.try_borrow_mut().unwrap().on_child_created()
    }

    pub(crate) fn on_child_dropped(&self) {
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

pub struct RcChunkAllocatorMetricsImpl<T: Debug, const N: usize> {
    children: usize,
    phantom: PhantomData<T>,
}

impl<T: Debug, const N: usize> RcChunkAllocatorMetricsImpl<T, N> {
    pub fn new() -> Self {
        Self {
            children: 0,
            phantom: PhantomData,
        }
    }

    pub fn memory_used(&self) -> usize {
        self.children * RcChunk::<T, N>::size()
    }

    pub fn on_child_created(&mut self) {
        self.children += 1;
    }

    pub fn on_child_dropped(&mut self) {
        self.children -= 1;
    }
}
