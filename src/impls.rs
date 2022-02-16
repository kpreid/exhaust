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
//! * [`core::cell::UnsafeCell`], because it does not implement [`Clone`].
//! * [`core::mem::ManuallyDrop`], because it would be a memory leak.
//! * [`std::io::ErrorKind`] and other explicitly non-exhaustive types.
//! * [`std::sync::Mutex`] and `RwLock`, which do not implement [`Clone`].
//!
//! [`Exhaust`]: crate::Exhaust

// Impls organized by the corresponding standard library module.
mod core_cell;
mod core_cmp;
mod core_convert;
mod core_future;
mod core_marker;
mod core_num;
mod core_option;
//  core::pin::Pin is handled separately for each pinnable smart pointer.
mod core_primitive;
pub use core_primitive::ExhaustArray;

#[cfg(feature = "alloc")]
mod alloc_impls;

#[cfg(feature = "std")]
mod std_impls;

// TODO: The following implementations might be missing:
//   Primitive tuples (except for unit)
//   core::iter::* (combinatorial explosion fun!)
//     Iterators for std library types *not* in core::iter
//   core::hash::*
//   core::lazy::* (not yet stabilized)
//   core::fmt::Alignment
//   core::fmt::Error (do we want to impl for Error types in general?)
//   core::mem::MaybeUninit
//   core::num::NonZero*
//   core::ops::{Bound, ControlFlow, Range*}
//   core::result::Result
//   core::sync::atomic::*
//   core::task::Poll
//   alloc::borrow::Cow (with the caveat of no Borroweds)
//   alloc::collections::{BinaryHeap, BTreeMap, BTreeSet}
//   std::collections::{HashMap, HashSet}
//   std::fs::{OpenOptions, Permissions} (avoid since it's non-exhaustive?)
//   std::io::{BufReader, BufWriter, Stdin, Stdout, Stderr, Sink}
