//! Implementations of [`Exhaust`] for standard library types.
//!
//! The public contents of this module are just the corresponding structs implementing
//! [`Iterator`]. These need to be public, but should mostly be considered an implementation
//! detail and not need to be used explicitly.
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
mod core_option;
mod core_sync;
//  core::pin::Pin is handled separately for each pinnable smart pointer.
mod core_primitive;
mod core_result;
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
