mod chunk;
mod chunk_allocator;
mod chunk_allocator_metrics;
mod rc;

pub use crate::chunk_allocator::RcChunkAllocator;
pub use crate::rc::Rc;

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct TestElem {
        index: usize,
    }

    impl TestElem {
        fn new(index: usize) -> Self {
            println!("new test elem={index}");
            Self { index }
        }
    }

    impl Drop for TestElem {
        fn drop(&mut self) {
            println!("dropping test elem={}", self.index);
        }
    }

    #[test]
    fn new() {
        const N: usize = 5;
        const OVERFILL: usize = 3;
        let alloc: RcChunkAllocator<TestElem, N> = RcChunkAllocator::new();
        let mut elems = vec![];
        println!(">>{}", alloc.memory_used());
        for i in 0..(OVERFILL * N) {
        println!("<>{}", alloc.memory_used());
            elems.push(alloc.alloc(TestElem::new(i)));
        }
        println!("<<{}", alloc.memory_used());
        drop(alloc);
        drop(elems);
        // TODO(kcza): fix leak shown by miri!
        // assert!(false);
    }
}
