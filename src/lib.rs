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

pub(crate) mod patterns;

pub mod impls;
/// Reexport for compatibility with v0.1.0;
/// new code should use [`impls::ExhaustArray`].
#[deprecated]
pub use impls::ExhaustArray;

mod convenience;
pub use convenience::*;

pub mod iteration;

/// Types that can be exhaustively iterated. That is, an iterator is available which
/// produces every possible value of this type.
///
/// When implementing this trait, take note of
/// [the requirements noted below in `exhaust()`](Self::exhaust) for a correct implementation.
///
/// Implementors must also implement [`Clone`]; this requirement is for the benefit of implementing
/// [`Exhaust`] for containers of this type. (Hopefully, a future version of the library may relax
/// this restriction as a breaking change, as it does prevent reasonable implementations for types
/// such as atomics.)
///
/// # Examples
///
/// Using [the derive macro](macro@Exhaust) to implement the trait:
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
///     fn exhaust() -> Self::Iter {
///         ExhaustAsciiLetter { next: 'A' }
///     }
/// }
///
/// #[derive(Clone)]
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
/// * Types which do not implement [`Clone`]:
///
///   * [`core::cell::UnsafeCell`]
///   * [`std::sync::Mutex`] and `RwLock`
///   * [`core::sync::atomic::Atomic*`](core::sync::atomic)
///
///   A future version of the library might relax the [`Clone`] bound,
///   but this is currently impossible.
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
/// * [`std::io::Stdout`] and other types whose sole use is in performing IO.

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
    /// * No duplicates: if [`Self: PartialEq`](PartialEq), then for any two items `a, b` produced
    ///   by the iterator, `a != b`.
    /// * Exhaustiveness: If [`Self: PartialEq`](PartialEq), then for every value `a` of type
    ///   `Self`, there is some element `b` of `Self::exhaust()` for which `a == b`,
    ///   unless it is the case that `a != a`.
    ///   If there is no `PartialEq` implementation, then follow the spirit of this rule anyway.
    /// * If there is any value `a` of type `Self` for which `a != a`, then [`Exhaust`]
    ///   must produce one or more such values (e.g. [`f32::NAN`]).
    /// * `exhaust()` does not panic, nor does the iterator it returns.
    /// * Purity/determinism: every call to `Self::exhaust()`, or [`Clone::clone()`] of a returned
    ///   iterator, should produce the same sequence of items.
    ///   (If this is not upheld, then derived implementations of [`Exhaust`] on types containing
    ///   this type will not behave consistently.)
    /// * The iterator has a finite length, that is feasible to actually reach.
    ///   (For example, [`u64`] does not implement [`Exhaust`].)
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
