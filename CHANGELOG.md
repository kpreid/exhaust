# Changelog

## Unreleased

### Added

* The derive macro now supports a configuration attribute `#[exhaust(factory_is_self)]`,
  which disables generation of a separate `Exhaust::Factory` type.
  This can be used to simplify the macro-generated code to improve build performance,
  but has particular requirements; see the derive macro’s documentation for details.

## 0.2.4 (2025-08-25)

### Added

* `derive(Exhaust)` now supports types with const generic parameters.

### Changed

* `derive(Exhaust)` generates simpler code to improve build performance (and readability).
  * Replaced use of `Option::unwrap()` with pattern matching.
  * Replaced recursion in enum iterator `next()` with iteration.
  * Factories for unit structs no longer include an inner type.
  * Removed superfluous `&& true`s.

### Fixed

* `derive(Exhaust)` will no longer produce spurious warnings when the type of a field is uninhabited.

## 0.2.3 (2025-08-24)

This release was published erroneously and is functionally equivalent to 0.2.2.

## 0.2.2 (2025-03-08)

### Added

* `impl Exhaust for ...`
  * `core::fmt::Alignment`
  * `core::fmt::Error`
  * `core::cell::OnceCell`
  * `core::ops::Bound`
  * `core::ops::ControlFlow`
  * `core::ops::RangeFrom`
  * `core::ops::RangeFull`
  * `core::ops::RangeTo`
  * `core::ops::RangeToInclusive`
  * `std::sync::OnceLock`
  * `std::sync::mpsc::RecvError`
  * `std::sync::mpsc::RecvTimeoutError`
  * `std::sync::mpsc::SendError`
  * `std::sync::mpsc::TryRecvError`
  * `std::sync::mpsc::TrySendError`

* The dependency on `itertools` can now use either version 0.14 or version 0.13.
  This has no effect on the functionality of `exhaust`.

## 0.2.1 (2024-09-26)

### Added

* `Cell<T>` implements `Exhaust` even when `T` does not implement `Copy`.
* Documentation contains examples for the `Exhaust` derive macro and `iteration::carry`.

### Changed

* The macro-generated types are now always named `Exhaust<your type name><some suffix>`.
  This makes it possible to reliably avoid name conflicts in the narrow case that they can happen,
  and is more systematic than the previous naming scheme.
* Explicitly `allow(nonstandard_style)` in macro generated code.
  Together with the above name change,
  this should prevent lint from the macro-generated code when using rust-analyzer.

## 0.2.0 (2024-09-18)

### Breaking: No `Clone` Requirement

The `Exhaust` trait no longer requires `Self: Clone`.
This allows it to be implemented for many more types, such as `Mutex` and `Atomic*`,
which can be *constructed* with arbitrary data but not cloned.

In order to support this, the `Exhaust` trait now separates iteration over the possible data
(which still must be cloneable) from construction of the final value.
The new definition of the trait is:

```rust
pub trait Exhaust: Sized {
    type Iter: FusedIterator<Item = Self::Factory> + Clone;
    type Factory: Clone;                                    // New

    // Required methods
    fn exhaust_factories() -> Self::Iter;                   // New
    fn from_factory(factory: Self::Factory) -> Self;        // New

    // Provided method
    fn exhaust() -> impl Iterator<Item = Self>;
}
```

The `Factory` is the cloneable data, so named since it is a means of constructing many `Self`.
Implementors must return an iterator which produces `Factory` instead of `Self`.
They may then perform the final construction in `from_factory()`.

Existing implementations may be migrated by adding `type Factory = Self`
and renaming `fn exhaust()` to `fn exhaust_factories()`.
However, it may be possible to simplify the iterator by moving some code into `fn from_factory()`.

### Added

* `impl Exhaust for ...`
    * `core::cell::UnsafeCell`
    * `core::sync::AtomicBool`
    * `core::sync::Atomic{U,I}{8,16,32}`
    * `alloc::borrow::Cow<'_ T>`, when `T` implements only `ToOwned` rather than `Clone`.
    * `std::io::BufReader`
    * `std::io::BufWriter`
    * `std::io::Chain`
    * `std::io::LineWriter`
    * `std::io::Repeat`
    * `std::io::Sink`
    * `std::io::Stderr`
    * `std::io::Stdin`
    * `std::io::Stdout`

* `exhaust::Iter<T>` is a single generic iterator for all `T: Exhaust`.
  It is now the type returned by `Exhaust::exhaust()`.

### Changed

* The minimum supported Rust version is now 1.80.
* **Breaking:** The derive macro `derive(Exhaust)` now hides its generated items.
  They can only be accessed through the trait implementation’s associated types.
* **Breaking:** `Exhaust::Iter` types now must implement `Debug` and `FusedIterator`.

### Removed

* `<Option<T> as Exhaust>::Iter` no longer implements `DoubleEndedIterator`.
  This might be added back in the future.
* Removed `exhaust::ExhaustArray` from the crate root.
* Removed `exhaust::brute_force_search()`.
* Removed the public module `exhaust::impls`.
  The iterators can only be accessed through the trait implementation’s associated types.

## 0.1.2 (2024-09-18)

There are no changes to functionality in this release.

* Improved documentation.
* Depends on `itertools` version 0.13 instead of 0.10.
* Added `rust-version` (minimum supported Rust version) information, chosen to be 1.60.
* `exhaust::ExhaustArray` is now marked as `#[deprecated]` (use `exhaust::impls::ExhaustArray` instead).

## 0.1.1 (2023-04-09)

### Added

* `impl Exhaust for ...`
    * Tuples of length up to 12
    * `core::cell::Cell`
    * `core::cell::RefCell`
    * `core::cmp::Ordering`
    * `core::future::Pending`
    * `core::future::Ready`
    * `core::hash::BuildHasherDefault`
    * `core::iter::Reverse`
    * `core::marker::PhantomPinned`
    * `core::num::FpCategory`
    * `core::num::NonZero*`
    * `core::result::Result`
    * `core::task::Poll`
    * `alloc::borrow::Cow`
    * `alloc::collections::BTreeMap`
    * `alloc::collections::BTreeSet`
        * Note: Does not produce lexicographic ordering.
    * `Pin<Box<T>>`
    * `std::collections::HashMap`
    * `std::collections::HashSet`
    * `std::io::Cursor`
    * `std::io::Empty`
    * `std::io::Sink`
* Documentation example of a custom `impl Exhaust`.

### Changed

* `derive(Exhaust)` generates more detailed documentation for the iterator.

### Meta

* Added license texts.
* Configured GitHub Actions CI.

## 0.1.0 (2022-02-14)

Initial public release.
