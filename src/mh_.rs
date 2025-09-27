//! Items used by the derive macro, placed in a single module with a short path for approximate
//! name hygiene and for concise output.

pub use core::fmt;
pub use core::iter::{FusedIterator, Iterator};
pub use {Clone, Default, None, Option, Some};

/// Convenience trait-alias for helping the derive macro be simpler and generate simpler code.
///
/// It includes `Clone + fmt::Debug` because those are the bounds required of factories,
/// and including them as supertrait bounds here allows avoiding reiterating them.
#[doc(hidden)]
pub trait ExhaustWithFactoryEqSelf: crate::Exhaust<Factory = Self> + Clone + fmt::Debug {}

impl<T> ExhaustWithFactoryEqSelf for T where T: crate::Exhaust<Factory = T> + Clone + fmt::Debug {}

// Trait methods cannot be `use`d, so define alias functions for them
#[must_use]
pub fn default<T: Default>() -> T {
    Default::default()
}
#[must_use]
pub fn next<I: Iterator>(iterator: &mut I) -> Option<I::Item> {
    iterator.next()
}
pub fn clone<T: Clone>(original: &T) -> T {
    original.clone()
}
pub fn peek<I: Iterator>(iter: &mut core::iter::Peekable<I>) -> Option<&I::Item> {
    iter.peek()
}
