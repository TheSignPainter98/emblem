use crate::context::Resource;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResourceLimit<T: Resource> {
    Unlimited,
    Limited(T),
}

impl<T: Resource> ResourceLimit<T> {
    pub(crate) fn contains(&self, amount: T) -> bool {
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
        assert!(unlimited.contains(Step(0)));

        let limited = ResourceLimit::Limited(Step(10));
        assert!(limited.contains(Step(0)));
        assert!(limited.contains(Step(9)));
        assert!(!limited.contains(Step(10)));
        assert!(!limited.contains(Step(100)));
    }
}
