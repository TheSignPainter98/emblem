use crate::chunk::Chunk;
use std::{fmt::Debug, marker::PhantomData};

pub(crate) struct ChunkAllocatorMetrics<T: Debug, const N: usize> {
    children: usize,
    phantom: PhantomData<T>,
}

impl<T: Debug, const N: usize> ChunkAllocatorMetrics<T, N> {
    pub(crate) fn new() -> Self {
        Self {
            children: 0,
            phantom: PhantomData,
        }
    }

    pub(crate) fn memory_used(&self) -> usize {
        self.children * Chunk::<T, N>::size()
    }

    pub(crate) fn on_child_created(&mut self) {
        self.children += 1;
    }

    pub(crate) fn on_child_dropped(&mut self) {
        self.children -= 1;
    }
}
