use core::ops;

use crate::patterns::{delegate_factory_and_iter, impl_singleton};
use crate::Exhaust;

impl<T: Exhaust> Exhaust for ops::Bound<T> {
    delegate_factory_and_iter!(remote::Bound<T>);

    fn from_factory(factory: Self::Factory) -> Self {
        match remote::Bound::from_factory(factory) {
            remote::Bound::Included(v) => ops::Bound::Included(v),
            remote::Bound::Excluded(v) => ops::Bound::Excluded(v),
            remote::Bound::Unbounded => ops::Bound::Unbounded,
        }
    }
}

impl<B: Exhaust, C: Exhaust> Exhaust for ops::ControlFlow<B, C> {
    delegate_factory_and_iter!(remote::ControlFlow<B, C>);

    fn from_factory(factory: Self::Factory) -> Self {
        match remote::ControlFlow::from_factory(factory) {
            remote::ControlFlow::Continue(v) => ops::ControlFlow::Continue(v),
            remote::ControlFlow::Break(v) => ops::ControlFlow::Break(v),
        }
    }
}

// Ranges with lower and upper bounds do not implement Exhaust because it is ambiguous whether
// reversed ranges (e.g. 10..5) should be supported: they are undesirable for “iterate over all
// powersets of T that are contiguous ranges” and desirable for “iterate over all values that
// could possibly be encountered”.
//
// The range types with one or zero endpoints do not have this problem.
//
// impl<T> !Exhaust for ops::Range<T> {}
// impl<T> !Exhaust for ops::RangeInclusive<T> {}

impl<T: Exhaust> Exhaust for ops::RangeFrom<T> {
    delegate_factory_and_iter!(T);

    fn from_factory(factory: Self::Factory) -> Self {
        ops::RangeFrom {
            start: T::from_factory(factory),
        }
    }
}

impl_singleton!([], ops::RangeFull);

impl<T: Exhaust> Exhaust for ops::RangeTo<T> {
    delegate_factory_and_iter!(T);

    fn from_factory(factory: Self::Factory) -> Self {
        ops::RangeTo {
            end: T::from_factory(factory),
        }
    }
}

impl<T: Exhaust> Exhaust for ops::RangeToInclusive<T> {
    delegate_factory_and_iter!(T);

    fn from_factory(factory: Self::Factory) -> Self {
        ops::RangeToInclusive {
            end: T::from_factory(factory),
        }
    }
}

/// Like the Serde “remote derive” pattern, we define a type imitating the real type
/// which the derive macro can process.
mod remote {
    #![allow(missing_debug_implementations)] // not actually public

    #[derive(crate::Exhaust)]
    pub enum ControlFlow<B, C> {
        Continue(C),
        Break(B),
    }

    #[derive(crate::Exhaust)]
    pub enum Bound<T> {
        Included(T),
        Excluded(T),
        Unbounded,
    }
}
