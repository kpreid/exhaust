use core::iter;

use crate::{Exhaust, Indexable};

impl Exhaust for core::convert::Infallible {
    type Iter = iter::Empty<core::convert::Infallible>;
    fn exhaust_factories() -> Self::Iter {
        iter::empty()
    }
    crate::patterns::factory_is_self!();
}

impl Indexable for core::convert::Infallible {
    const VALUE_COUNT: usize = 0;

    #[mutants::skip]
    fn to_index(value: &Self) -> usize {
        match *value {}
    }

    #[track_caller]
    #[mutants::skip]
    fn from_index(_: usize) -> Self {
        panic!("core::convert::Infallible has no valid indices")
    }
}
