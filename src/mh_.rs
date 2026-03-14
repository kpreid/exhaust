//! Items used by the derive macro, placed in a single module with a short path for approximate
//! name hygiene and for concise output.

pub use core::fmt;
pub use core::iter::{FusedIterator, Iterator, Peekable};
pub use core::primitive::usize;
pub use {Clone, Default, None, Option, Some};

/// Types which are valid as fields of `Exhaust<Factory = Self>` implementations.
/// Such types must both implement [`Exhaust`] and also the traits required of a factory type,
/// i.e. `Clone + fmt::Debug`
///
/// This trait alias helps the derive macro be simpler and generate simpler code.
#[doc(hidden)]
pub trait ExhaustAndFactoryish: crate::Exhaust + Clone + fmt::Debug {}

impl<T> ExhaustAndFactoryish for T where T: crate::Exhaust + Clone + fmt::Debug {}

// Trait methods cannot be `use`d, so define alias functions for them
#[must_use]
pub fn default<T: Default>() -> T {
    Default::default()
}
#[must_use]
pub fn next<I: Iterator>(iterator: &mut I) -> Option<I::Item> {
    iterator.next()
}
pub fn size_hint<I: Iterator>(iterator: &I) -> (usize, Option<usize>) {
    iterator.size_hint()
}
pub fn clone<T: Clone>(original: &T) -> T {
    original.clone()
}
pub fn peek<I: Iterator>(iter: &mut Peekable<I>) -> Option<&I::Item> {
    iter.peek()
}

/// Peekable iterator over factories of `T`.
pub type Pf<T> = Peekable<<T as crate::Exhaust>::Iter>;
/// Peekable iterator over values of `T`.
pub type Pv<T> = Peekable<crate::Iter<T>>;

/// Constructs [`Pf`].
pub fn pf_iter<T: crate::Exhaust>() -> Pf<T> {
    T::exhaust_factories().peekable()
}

/// Constructs [`Pv`].
pub fn pv_iter<T: crate::Exhaust>() -> Pv<T> {
    crate::Iter::<T>::default().peekable()
}
