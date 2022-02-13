//! Assistance for implementing exhaustive iterators.
//!
//! These functions were found to be repeatedly useful within the built-in implementations,
//! and are provided publicly in the expectation that they will have more uses.

use core::iter::Peekable;

use crate::Exhaust;

/// Construct a [`Peekable`] exhaustive iterator.
///
/// Peekable iterators are useful for iterating over the product of multiple iterators.
pub fn peekable_exhaust<T: Exhaust>() -> Peekable<T::Iter> {
    T::exhaust().peekable()
}

/// Perform “carry” within a pair of peekable iterators.
///
/// That is, if `low` is exhausted, advance `high`, and replace `low`
/// with a fresh iterator from the factory function.
///
/// Returns whether a carry occurred.
pub fn carry<I, J, F>(high: &mut Peekable<I>, low: &mut Peekable<J>, factory: F) -> bool
where
    I: Iterator,
    J: Iterator,
    F: FnOnce() -> Peekable<J>,
{
    if low.peek().is_none() {
        *low = factory();
        high.next();
        true
    } else {
        false
    }
}
