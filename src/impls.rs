//! Implementations of [`Exhaust`] for standard library types.
//!
//! The public contents of this module are just the corresponding structs implementing
//! [`Iterator`]. These need to be public, but should mostly be considered an implementation
//! detail and not need to be used explicitly.
//!
//! The following primitive or standard library types do not implement [`Exhaust`] for
//! particular reasons:
//!
//! * References, because there's nowhere to stash the referent.
//!   (This could be changed for small finite types, like `&bool`, but those are the same
//!   sort of types which are unlikely to be used by reference.)
//! * Pointers, for the same reason as references (and we could generate invalid pointers,
//!   but that would be almost certainly pointless).
//! * [`u64`], [`i64`], and [`f64`], because they are too large to feasibly exhaust.
//! * Types which do not implement [`Clone`]:
//!
//!   * [`core::cell::UnsafeCell`]
//!   * [`std::sync::Mutex`] and `RwLock`
//!   * [`core::sync::atomic::Atomic*`](core::sync::atomic)
//!
//!   A future version of the library might relax the [`Clone`] bound,
//!   but this is currently impossible.
//! * Containers that permit duplicate items, and can therefore be unboundedly large:
//!   * [`alloc::vec::Vec`]
//!   * [`alloc::collections::VecDeque`]
//!   * [`alloc::collections::LinkedList`]
//!   * [`alloc::collections::BinaryHeap`]
//!
//! * [`core::mem::ManuallyDrop`], because it would be a memory leak.
//! * [`core::mem::MaybeUninit`], because it is not useful to obtain a `MaybeUninit<T>`
//!   value without knowing whether it is initialized, and if they are to be all
//!   initialized, then `T::exhaust()` is just as good.
//! * [`core::ops::Range*`](core::ops), because it is ambiguous whether inverted (start > end)
//!   ranges should be generated.
//! * [`std::io::ErrorKind`] and other explicitly non-exhaustive types.
//! * [`std::io::Stdout`] and other types whose sole use is in performing IO.
//!
//! [`Exhaust`]: crate::Exhaust

// Impls organized by the corresponding standard library module.
mod core_cell;
mod core_cmp;
mod core_convert;
mod core_future;
mod core_hash;
mod core_marker;
mod core_num;
pub use core_num::*;
mod core_option;
//  core::pin::Pin is handled separately for each pinnable smart pointer.
mod core_primitive;
pub use core_primitive::*;
mod core_result;
pub use core_result::*;
mod core_task;

#[cfg(feature = "alloc")]
mod alloc_impls;

#[cfg(feature = "std")]
mod std_impls;

// TODO: The following implementations might be missing:
//   core::iter::* (combinatorial explosion fun!)
//     Iterators for std library types *not* in core::iter
//   `OnceCell` & `OnceLock`
//   core::fmt::Alignment
//   core::fmt::Error (do we want to impl for Error types in general?)
//   core::ops::{Bound, ControlFlow, Range*}
