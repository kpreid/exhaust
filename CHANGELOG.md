# Changelog

## Unreleased 0.2.0 (date TBD)

### Changed

* The minimum supported Rust version is now 1.80.

### Removed

* `<Option<T> as Exhaust>::Iter` no longer implements `DoubleEndedIterator`.
  This might be added back in the future.
* Removed `exhaust::ExhaustArray` from the crate root.
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
