#![no_std]

//! This crate provides the [`Exhaust`] trait and derive macro, which allow iterating over
//! all values of a given type.

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub use exhaust_macros::Exhaust;

mod impls;
pub use impls::*;

mod convenience;
pub use convenience::*;

#[cfg(test)]
mod tests;

/// Types that can be exhaustively iterated. That is, an iterator is available which
/// produces every possible value of this type.
///
/// Implementors must also implement [`Clone`], because it is useful for the purpose of
/// implementing [`Exhaust`] on types containing this type. This should never be a
/// significant restriction since a type implementing [`Exhaust`] implies that every
/// instance can be derived from pure data (“the Nth element of `T::exhaust()`”).
///
/// This trait is not an `unsafe trait`, and as such, no soundness property should rest
/// on implementations having any of the above properties.
pub trait Exhaust: Clone {
    /// Type of iterator returned by [`Self::exhaust()`].
    ///
    /// Note: While it is necessary for this type to be exposed, an implementation of
    /// [`Exhaust`] changing this to another type should not be considered a breaking
    /// change, as long as it still has the same iterator properties (e.g.
    /// [`ExactSizeIterator`]).
    type Iter: Iterator<Item = Self> + Clone;

    /// Returns an iterator over all values of this type.
    ///
    /// Implementations should have the following properties:
    ///
    /// * For any two items `a, b` produced by the iterator, `a != b`.
    /// * For every value `a` of type `Self`, there is some element `b` of `Self::exhaust()`
    ///   for which `a == b`, unless it is the case that `a != a`.
    /// * If there is any value `a` of type `Self` for which `a != a`, then [`Exhaust`]
    ///   must produce one or more such values.
    /// * `exhaust()` does not panic, nor does the iterator it returns.
    /// * The iterator has a finite length, that is feasible to actually reach.
    fn exhaust() -> Self::Iter;
}
