# Changelog

## Unreleased

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
    * `std::collections::HashSet`
    * `std::io::Cursor`
* Documentation example of a custom `impl Exhaust`.

### Changed

* `derive(Exhaust)` generates more detailed documentation for the iterator.

### Meta

* Added license texts.
* Configured GitHub Actions CI.

## 0.1.0 (2022-02-14)

Initial public release.
