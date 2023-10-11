use crate::context::Resource;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResourceLimit<T: Resource> {
    Unlimited,
    Limited(T),
}

impl<T: Resource> ResourceLimit<T> {
    /// Returns whether the resource limit is less than the given amount.
    pub(crate) fn lt(&self, amount: T) -> bool {
        match self {
            Self::Unlimited => true,
            Self::Limited(l) => amount < *l,
        }
    }
}

impl<T: Resource> Default for ResourceLimit<T> {
    fn default() -> Self {
        Self::Limited(T::default_limit())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::context::{Iteration, Memory, Step};

    #[test]
    fn default() {
        test_default::<Step>();
        test_default::<Memory>();
        test_default::<Iteration>();
    }

    fn test_default<T: Resource>() {
        match ResourceLimit::<T>::default() {
            ResourceLimit::Unlimited => panic!("default is unlimited"),
            ResourceLimit::Limited(limit) => {
                assert!(limit > T::default(), "limit must be greater than default")
            }
        }
    }

    #[test]
    fn contains() {
        let unlimited = ResourceLimit::Unlimited;
        assert!(unlimited.lt(Step(0)));

        let limited = ResourceLimit::Limited(Step(10));
        assert!(limited.lt(Step(0)));
        assert!(limited.lt(Step(9)));
        assert!(!limited.lt(Step(10)));
        assert!(!limited.lt(Step(100)));
    }
}
