//! Assistance for implementing exhaustive iterators.
//!
//! These functions were found to be repeatedly useful within the built-in implementations,
//! and are provided publicly in the expectation that they will have more uses.

use core::iter::Peekable;
use core::{fmt, iter};

use crate::Exhaust;

/// Convenience alias for a Peekable Exhaustive Iterator, frequently used in iterator
/// implementations.
pub type Pei<T> = Peekable<<T as Exhaust>::Iter>;

/// Construct a [`Peekable`] exhaustive factory iterator.
///
/// Peekable iterators are useful for iterating over the product of multiple iterators.
pub fn peekable_exhaust<T: Exhaust>() -> Pei<T> {
    T::exhaust_factories().peekable()
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

/// Given an iterator and a function of its elements that yields an iterator,
/// produce tuples of the two iterators' results.
#[derive(Clone)]
#[doc(hidden)] // Public because exposed as an iterator type. Not yet recommended for use.
pub struct FlatZipMap<I: Iterator, J: Iterator, O> {
    outer_iterator: I,
    inner: Option<(I::Item, J)>,
    iter_fn: fn(&I::Item) -> J,
    output_fn: fn(I::Item, J::Item) -> O,
}

impl<I, J, O> fmt::Debug for FlatZipMap<I, J, O>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug,
    J: Iterator + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlatZipMap")
            .field("outer_iterator", &self.outer_iterator)
            .field("outer_item", &self.inner.as_ref().map(|i| &i.0))
            .field("inner_iterator", &self.inner.as_ref().map(|i| &i.1))
            .finish_non_exhaustive()
    }
}

impl<I, J, O> FlatZipMap<I, J, O>
where
    I: Iterator,
    J: Iterator,
{
    #[cfg_attr(not(feature = "std"), allow(dead_code))]
    pub(crate) fn new(
        outer_iterator: I,
        iter_fn: fn(&I::Item) -> J,
        output_fn: fn(I::Item, J::Item) -> O,
    ) -> Self {
        Self {
            outer_iterator,
            inner: None,
            iter_fn,
            output_fn,
        }
    }
}

impl<I, J, O> Iterator for FlatZipMap<I, J, O>
where
    I: Iterator,
    I::Item: Clone,
    J: Iterator,
{
    type Item = O;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner {
                Some((ref i_item, ref mut j_iter)) => {
                    if let Some(j_item) = j_iter.next() {
                        return Some((self.output_fn)(i_item.clone(), j_item));
                    }
                    // If no items, try the outer iter.
                    self.inner = None;
                }
                None => match self.outer_iterator.next() {
                    Some(i_item) => {
                        let j_iter = (self.iter_fn)(&i_item);
                        self.inner = Some((i_item, j_iter));
                    }
                    None => return None,
                },
            }
        }
    }
}

impl<I, J, O> iter::FusedIterator for FlatZipMap<I, J, O>
where
    I: Iterator,
    I::Item: Clone,
    J: Iterator,
{
}
