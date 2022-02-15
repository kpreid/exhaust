#![no_std]

//! This crate provides the [`Exhaust`] trait and derive macro, which allow iterating over
//! all values of a given type.

#![forbid(rust_2018_idioms)]
#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(clippy::cast_lossless)]
#![warn(clippy::exhaustive_enums)]
#![warn(clippy::exhaustive_structs)]
#![warn(clippy::pedantic)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub use exhaust_macros::Exhaust;

mod impls;
pub use impls::*;

mod convenience;
pub use convenience::*;

pub mod iteration;

/// Types that can be exhaustively iterated. That is, an iterator is available which
/// produces every possible value of this type.
///
/// Implementors must also implement [`Clone`], because it is useful for the purpose of
/// implementing [`Exhaust`] on types containing this type. This should never be a
/// significant restriction since a type implementing [`Exhaust`] implies that every
/// instance can be derived from pure data (“the Nth element of `T::exhaust()`”).
///
/// # Examples
///
/// ```
/// use exhaust::Exhaust;
///
/// #[derive(Clone, PartialEq, Debug, Exhaust)]
/// struct Foo {
///     a: bool,
///     b: Bar,
/// }
///
/// #[derive(Clone, PartialEq, Debug, Exhaust)]
/// enum Bar {
///     One,
///     Two(bool),
/// }
///
/// assert_eq!(
///     Foo::exhaust().collect::<Vec<Foo>>(),
///     vec![
///         Foo { a: false, b: Bar::One },
///         Foo { a: false, b: Bar::Two(false) },
///         Foo { a: false, b: Bar::Two(true) },
///         Foo { a: true, b: Bar::One },
///         Foo { a: true, b: Bar::Two(false) },
///         Foo { a: true, b: Bar::Two(true) },
///     ],
/// );
/// ```
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
    /// * Purity/determinism: every call to `Self::exhaust()`, or [`Clone::clone()`] of a returned
    ///   iterator, should produce the same sequence of items.
    /// * The iterator has a finite length, that is feasible to actually reach.
    ///
    /// [`Exhaust`] is not an `unsafe trait`, and as such, no soundness property should rest
    /// on implementations having any of the above properties unless the particular implementation
    /// guarantees them.
    ///
    /// The following further properties are recommended when feasible:
    ///
    /// * If `Self: Ord`, then the items are sorted in ascending order.
    fn exhaust() -> Self::Iter;
}
