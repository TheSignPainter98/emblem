mod rc;
mod rc_chunk;
mod rc_chunk_allocator;
mod rc_chunk_allocator_metrics;

pub use crate::rc::Rc;
pub use crate::rc_chunk_allocator::RcChunkAllocator;

#[cfg(test)]
mod test {
    use super::*;
    use std::{cell::RefCell, rc::Rc as StdRc};

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
    fn alloc() {
        const N: usize = 10;
        let alloc = RcChunkAllocator::<TestElem, N>::new();
        assert_eq!(
            alloc.memory_used(),
            0,
            "initial memory should be zero, got {}",
            alloc.memory_used()
        );
        let increment;
        {
            let mut nums = 1..;

            let mut elems = vec![];
            for _ in 0..N {
                elems.push(alloc.alloc(TestElem::new(nums.next().unwrap())));
            }
            increment = alloc.memory_used();

            {
                const TRANSIENT_REPS: usize = 25;
                let mut tmp = vec![];
                for _ in 0..TRANSIENT_REPS {
                    for _ in 0..N {
                        tmp.push(alloc.alloc(TestElem::new(nums.next().unwrap())))
                    }
                }
                let expected = (1 + TRANSIENT_REPS) * increment;
                assert_eq!(
                    alloc.memory_used(),
                    expected,
                    "expected {expected} memory used, got {}",
                    alloc.memory_used()
                );
            }

            const PERSISTENT_REPS: usize = 100;
            for _ in 0..PERSISTENT_REPS {
                for _ in 0..N {
                    elems.push(alloc.alloc(TestElem::new(nums.next().unwrap())));
                }
            }

            let expected = (1 + PERSISTENT_REPS) * increment;
            assert_eq!(
                alloc.memory_used(),
                expected,
                "expected {expected} memory used, got {}",
                alloc.memory_used()
            );
        }
        assert_eq!(
            alloc.memory_used(),
            increment,
            "final memory should be {increment}, got {}",
            alloc.memory_used()
        );
    }

    #[test]
    fn clean() {
        const N: usize = 10;
        let alloc = RcChunkAllocator::<TestElem, N>::new();
        assert!(alloc.is_clean(), "fresh allocator reported as dirty");

        let mut elems = vec![];
        for i in 0..2 * N {
            elems.push(alloc.alloc(TestElem::new(i)));
        }
        assert!(!alloc.is_clean(), "dirty allocator reported as clean");
        alloc.clean();
        assert!(alloc.is_clean(), "cleaned allocator reported as dirty");
    }

    #[test]
    fn elems_dropped() {
        const N: usize = 10;

        #[derive(Debug)]
        struct ToDrop {
            dropped: StdRc<RefCell<bool>>,
        }

        impl Drop for ToDrop {
            fn drop(&mut self) {
                *self.dropped.borrow_mut() = true;
            }
        }

        for tests in (3 * N - 1)..=(3 * N + 1) {
            let flags: Vec<_> = (0..tests)
                .map(|_| StdRc::new(RefCell::new(false)))
                .collect();
            {
                let alloc = RcChunkAllocator::<ToDrop, N>::new();
                flags.iter().for_each(|flag| {
                    alloc.alloc(ToDrop {
                        dropped: flag.clone(),
                    });
                });
                assert!(
                    flags.iter().any(|f| *f.borrow()) && flags.iter().any(|f| !*f.borrow()),
                    "expected some drop functions to have been run but not others: {flags:?}"
                );
            }
            assert!(
                flags.iter().all(|f| *f.borrow()),
                "some drop functions were not run {:?}",
                flags
            );
        }
    }
}
