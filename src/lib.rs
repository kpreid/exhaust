#![no_std]

//! This crate provides the [`Exhaust`] trait and derive macro, which allow iterating over
//! all values of a given type.
//!
//! # Package features
//!
//! All features are enabled by default.
//! If you set `default-features = false`, `exhaust` becomes `no_std` compatible.
//! The `alloc` and `std` features add `Exhaust` implementations for
//! the corresponding standard library crates.

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

/// Allows the derive macro to be used internally.
extern crate self as exhaust;

// -------------------------------------------------------------------------------------------------

use core::fmt;
use core::iter::FusedIterator;

// -------------------------------------------------------------------------------------------------

pub(crate) mod patterns;

mod impls;

mod convenience;
pub use convenience::*;

pub mod iteration;

#[cfg(doctest)]
pub mod test_compile_fail;

// -------------------------------------------------------------------------------------------------

/// Types that can be exhaustively iterated. That is, an iterator is available which
/// produces every possible value of this type.
///
/// # Properties
///
/// Implementations should have the following properties:
///
/// * No duplicates: if [`Self: PartialEq`](PartialEq), then for any two items `a, b` produced
///   by the iterator, `a != b`.
/// * Exhaustiveness: If [`Self: PartialEq`](PartialEq), then for every value `a` of type
///   `Self`, there is some element `b` of `Self::exhaust()` for which `a == b`,
///   unless it is the case that `a != a`.
///   If there is no `PartialEq` implementation, then follow the spirit of this rule anyway.
/// * If there is any value `a` of type `Self` for which `a != a`, then [`Exhaust`]
///   must produce one or more such values (e.g. [`f32::NAN`]).
/// * `exhaust()` does not panic, nor does the iterator it returns,
///   except in the event that memory allocation fails.
/// * Purity/determinism: every call to `Self::exhaust()`, or [`Clone::clone()`] of a returned
///   iterator or factory, should produce the same sequence of items.
///   (If this is not upheld, then derived implementations of [`Exhaust`] on types containing
///   this type will not behave consistently.)
/// * The iterator has a finite length, that is feasible to actually reach.
///   (For example, [`u64`] does not implement [`Exhaust`].)
///
/// The following further properties are recommended when feasible:
///
/// * If `Self: Ord`, then the items are sorted in ascending order.
///
/// [`Exhaust`] is not an `unsafe trait`, and as such, no soundness property should rest
/// on implementations having any of the above properties unless the particular implementation
/// guarantees them.
///
/// # Examples
///
/// Using [the derive macro](macro@Exhaust) to implement the trait:
///
/// ```
/// use exhaust::Exhaust;
///
/// #[derive(PartialEq, Debug, Exhaust)]
/// struct Foo {
///     a: bool,
///     b: Bar,
/// }
///
/// #[derive(PartialEq, Debug, Exhaust)]
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
///
/// Writing a manual implementation of `Exhaust`:
///
/// ```
/// use exhaust::Exhaust;
///
/// #[derive(Clone)]
/// struct AsciiLetter(char);
///
/// impl Exhaust for AsciiLetter {
///     type Iter = ExhaustAsciiLetter;
///
///     // We could avoid needing to `derive(Clone)` by using `char` as the factory, but
///     // if we did that, then `from_factory()` must check its argument for validity.
///     type Factory = Self;
///
///     fn exhaust_factories() -> Self::Iter {
///         ExhaustAsciiLetter { next: 'A' }
///     }
///
///     fn from_factory(factory: Self::Factory) -> Self {
///         factory
///     }
/// }
///
/// #[derive(Clone)]  // All `Exhaust::Iter`s must implement `Clone`.
/// struct ExhaustAsciiLetter {
///     next: char
/// }
///
/// impl Iterator for ExhaustAsciiLetter {
///     type Item = AsciiLetter;
///
///     fn next(&mut self) -> Option<Self::Item> {
///         match self.next {
///             'A'..='Y' | 'a'..='z' => {
///                 let item = self.next;
///                 self.next = char::from_u32(self.next as u32 + 1).unwrap();
///                 Some(AsciiLetter(item))
///             }
///             'Z' => {
///                 self.next = 'a';
///                 Some(AsciiLetter('Z'))
///             }
///             '{' => None,  // ('z' + 1)
///             _ => unreachable!(),
///         }
///     }
/// }
/// impl std::iter::FusedIterator for ExhaustAsciiLetter {}
///
/// assert_eq!(
///     AsciiLetter::exhaust().map(|l| l.0).collect::<String>(),
///     String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"),
/// );
/// ```
///
/// # Excluded Types
///
/// The following primitive or standard library types **do not implement** [`Exhaust`] for
/// particular reasons:
///
/// * References, because there's nowhere to stash the referent.
///   (This could be changed for small finite types, like `&bool`, but those are the same
///   sort of types which are unlikely to be used by reference.)
/// * Pointers, for the same reason as references (and we could generate invalid pointers,
///   but that would be almost certainly pointless).
/// * [`u64`], [`i64`], and [`f64`], because they are too large to feasibly exhaust.
/// * Containers that permit duplicate items, and can therefore be unboundedly large:
///   * [`alloc::vec::Vec`]
///   * [`alloc::collections::VecDeque`]
///   * [`alloc::collections::LinkedList`]
///   * [`alloc::collections::BinaryHeap`]
///
/// * [`core::mem::ManuallyDrop`], because it would be a memory leak.
/// * [`core::mem::MaybeUninit`], because it is not useful to obtain a `MaybeUninit<T>`
///   value without knowing whether it is initialized, and if they are to be all
///   initialized, then `T::exhaust()` is just as good.
/// * [`core::ops::Range*`](core::ops), because it is ambiguous whether inverted (start > end)
///   ranges should be generated.
/// * [`std::io::ErrorKind`] and other explicitly non-exhaustive types.
pub trait Exhaust: Sized {
    /// Iterator type returned by [`Self::exhaust_factories()`].
    /// See the trait documentation for what properties this iterator should have.
    ///
    /// <div class="warning">
    ///
    /// Note: While it is necessary for this type to be exposed, an implementation of
    /// [`Exhaust`] changing to another iterator type should not be considered a breaking
    /// change, as long as it still has the same iterator properties (e.g.
    /// [`ExactSizeIterator`]); it should be treated as an implementation detail.
    ///
    /// </div>
    type Iter: core::iter::FusedIterator<Item = Self::Factory> + Clone;

    /// Data which can be used to construct `Self`.
    ///
    /// The difference between `Self` and `Self::Factory` is that the `Factory` must
    /// implement [`Clone`] even if `Self` does not. In the case where `Self` does implement
    /// [`Clone`], this can be set equal to `Self`.
    ///
    /// Factories are useful for implementing [`Exhaust`] for other types that contain this type,
    /// when this type does not implement [`Clone`],
    /// since the process will often require producing clones.
    ///
    /// <div class="warning">
    ///
    /// Note: While it is necessary for this type to be exposed, an implementation of
    /// [`Exhaust`] changing to another factory type should not be considered a breaking
    /// change; it should be treated as an implementation detail, unless otherwise documented.
    ///
    /// </div>
    type Factory: Clone;

    /// Returns an iterator over all values of this type.
    ///
    /// See the trait documentation for what properties this iterator should have.
    ///
    /// This function is equivalent to `Self::exhaust_factories().map(Self::from_factory)`.
    /// Implementors should not override it.
    #[must_use]
    fn exhaust() -> Iter<Self> {
        Iter::default()
    }

    /// Returns an iterator over [factories](Self::Factory) for all values of this type.
    ///
    /// Implement this function to implement the trait. Call this function when implementing an
    /// [`Exhaust::Iter`] iterator for a type that contains this type.
    ///
    /// See the trait documentation for what properties this iterator should have.
    #[must_use]
    fn exhaust_factories() -> Self::Iter;

    /// Construct a concrete value of this type from a `Self::Factory` value produced by
    /// its `Self::Iter`.
    ///
    /// <div class="warning">
    ///
    /// Caution: While this function is meant to be used only with values produced by the iterator,
    /// this cannot be enforced; therefore, make sure it cannot bypass any invariants that
    /// the type might have.
    ///
    /// It is acceptable for this function to panic if it is
    /// given a value that [`Self::Iter`] is unable to produce.
    ///
    /// </div>
    #[must_use]
    fn from_factory(factory: Self::Factory) -> Self;
}

/// Derive macro generating an impl of the trait [`Exhaust`].
///
/// This macro may be applied to `struct`s and `enum`s, but not `union`s.
///
/// The generated iterator and factory types will be unnameable except through
/// the trait implementation’s associated types.
///
/// The generated iterator implements [`FusedIterator`],
/// but not [`DoubleEndedIterator`] or [`ExactSizeIterator`].
/// It does not currently override any of the optional iterator methods such as
/// [`Iterator::size_hint()`].
pub use exhaust_macros::Exhaust;

// -------------------------------------------------------------------------------------------------

/// Iterator over all values of any type that implements [`Exhaust`].
///
/// It may be obtained with [`T::exhaust()`](Exhaust::exhaust) or [`Default::default()`].
pub struct Iter<T: Exhaust>(<T as Exhaust>::Iter);

impl<T: Exhaust> Default for Iter<T> {
    #[inline]
    fn default() -> Self {
        Self(T::exhaust_factories())
    }
}

impl<T: Exhaust> Iterator for Iter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(T::from_factory)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.fold(init, |state, item_factory| {
            f(state, T::from_factory(item_factory))
        })
    }
}

impl<T: Exhaust<Iter: DoubleEndedIterator>> DoubleEndedIterator for Iter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(T::from_factory)
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.rfold(init, |state, item_factory| {
            f(state, T::from_factory(item_factory))
        })
    }
}

impl<T: Exhaust> FusedIterator for Iter<T> {
    // Note: This is only correct because of the `FusedIterator` bound on `Exhaust::Iter`.
    // Otherwise we would have to add a `T::Iter: FusedIterator` bound here too.
}

impl<T: Exhaust<Iter: ExactSizeIterator>> ExactSizeIterator for Iter<T> {}

impl<T: Exhaust> Clone for Iter<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Exhaust<Iter: Copy>> Copy for Iter<T> {}

impl<T: Exhaust<Iter: fmt::Debug>> fmt::Debug for Iter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("exhaust::Iter").field(&self.0).finish()
    }
}
