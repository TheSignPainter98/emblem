use crate::{rc::Rc, rc_chunk::RcChunk, rc_chunk_allocator_metrics::RcChunkAllocatorMetrics};
use std::{cell::RefCell, fmt::Debug, rc::Rc as StdRc};

pub struct RcChunkAllocator<T: Debug, const N: usize> {
    inner: StdRc<RefCell<RcChunkAllocatorImpl<T, N>>>,
    metrics: RcChunkAllocatorMetrics<T, N>,
}

impl<T: Debug, const N: usize> RcChunkAllocator<T, N> {
    pub fn new() -> Self {
        Self::check();
        Self {
            inner: StdRc::new(RefCell::new(RcChunkAllocatorImpl::new())),
            metrics: RcChunkAllocatorMetrics::new(),
        }
    }

    pub fn is_clean(&self) -> bool {
        self.inner.try_borrow().unwrap().is_clean()
    }

    pub fn clean(&self) {
        self.inner.try_borrow_mut().unwrap().clean();
    }

    pub fn alloc(&self, t: T) -> Rc<T, N> {
        self.inner.try_borrow_mut().unwrap().alloc(self, t)
    }

    fn metrics(&self) -> &RcChunkAllocatorMetrics<T, N> {
        &self.metrics
    }
}

impl<T: Debug, const N: usize> Default for RcChunkAllocator<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug, const N: usize> Check for RcChunkAllocator<T, N> {
    const VALID: () = assert!(N > 0, "chunk size parameter must be greater than zero");
}

impl<T: Debug, const N: usize> Clone for RcChunkAllocator<T, N> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

struct RcChunkAllocatorImpl<T: Debug, const N: usize> {
    chunk: Option<RcChunk<T, N>>,
}

impl<T: Debug, const N: usize> RcChunkAllocatorImpl<T, N> {
    fn new() -> Self {
        Self { chunk: None }
    }

    fn alloc(&mut self, parent: &RcChunkAllocator<T, N>, t: T) -> Rc<T, N> {
        if self.chunk.is_none() {
            self.refresh(parent)
        }

        let chunk = self.chunk.as_ref().unwrap();
        match chunk.try_alloc(t) {
            Ok(index) => Rc::new(chunk.clone(), index),
            Err(t) => {
                self.refresh(parent);
                let index = self
                    .chunk
                    .as_ref()
                    .expect("internal error: refresh did not create fresh chunk")
                    .try_alloc(t)
                    .expect("internal error: fresh chunk failed to allocate");
                Rc::new(self.chunk.as_ref().unwrap().clone(), index)
            }
        }
    }

    fn is_clean(&self) -> bool {
        self.chunk.is_none()
    }

    fn clean(&mut self) {
        self.chunk = None;
    }

    fn refresh(&mut self, parent: &RcChunkAllocator<T, N>) {
        parent.metrics().on_child_created();
        self.chunk = Some(RcChunk::new(parent.metrics().to_owned()));
    }
}

trait Check {
    const VALID: ();

    #[allow(clippy::let_unit_value)]
    fn check() {
        _ = Self::VALID;
    }
}
