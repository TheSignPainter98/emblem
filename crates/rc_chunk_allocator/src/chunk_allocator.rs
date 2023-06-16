use crate::{rc::Rc, chunk::Chunk};
use std::{rc::Rc as StdRc, fmt::Debug, cell::RefCell};

pub struct RcChunkAllocator<T: Debug, const N: usize> {
    inner: StdRc<RefCell<RcChunkAllocatorImpl<T, N>>>,
}

impl<T: Debug, const N: usize> RcChunkAllocator<T, N> {
    pub fn new() -> Self {
        Self::check();
        Self {
            inner: StdRc::new(RefCell::new(RcChunkAllocatorImpl::new())),
        }
    }

    pub fn is_clean(&self) -> bool {
        self.inner.try_borrow().unwrap().is_clean()
    }

    pub fn clean(&self) {
        self.inner.try_borrow_mut().unwrap().clean(&self);
    }

    pub fn alloc(&self, t: T) -> Rc<T, N> {
        self.inner.try_borrow_mut().unwrap().alloc(&self, t)
    }

    pub fn refresh(&self) {
        self.inner.try_borrow_mut().unwrap().clean(self);
    }

    pub fn memory_used(&self) -> usize {
        self.inner.try_borrow().unwrap().memory_used()
    }

    pub(crate) fn on_child_dropped(&mut self) {
        // println!("on_child_dropped");
        // self.inner.try_borrow_mut().unwrap().on_child_dropped()
    }
}

impl<T: Debug, const N: usize> Check for RcChunkAllocator<T, N> {
    const VALID: () = assert!(N > 0, "chunk size parameter must be greater than zero");
}

impl<T: Debug, const N: usize> Clone for RcChunkAllocator<T, N> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct RcChunkAllocatorImpl<T: Debug, const N: usize> {
    children: usize,
    chunk: Option<Chunk<T, N>>,
}

impl<T: Debug, const N: usize> RcChunkAllocatorImpl<T, N> {
    fn new() -> Self {
        Self {
            children: 0,
            chunk: None,
        }
    }

    fn alloc(&mut self, parent: &RcChunkAllocator<T, N>, t: T) -> Rc<T, N> {
        if self.chunk.is_none() {
            self.clean(parent)
        }

        match self.chunk.as_ref().unwrap().try_alloc(t) {
            Ok(index) => Rc::new(self.chunk.as_ref().unwrap().clone(), index),
            Err(t) => {
                self.clean(parent);
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
        self.chunk.as_ref().is_some_and(Chunk::is_empty)
    }

    fn clean(&mut self, parent: &RcChunkAllocator<T, N>) {
        self.children += 1;
        self.chunk = Some(Chunk::new(parent.to_owned()));
    }

    /// Approximate the amount of memory used by the top level of child constructs
    fn memory_used(&self) -> usize {
        self.children * Chunk::<T, N>::size()
    }

    pub(crate) fn on_child_dropped(&mut self) {
        self.children -= 1;
    }
}

pub trait Check {
    const VALID: ();

    fn check() {
        _ = Self::VALID;
    }
}
