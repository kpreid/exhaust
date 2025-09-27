use core::{fmt, iter};

use crate::iteration::{carry, peekable_exhaust};
use crate::{Exhaust, Indexable};

impl<T: Exhaust, const N: usize> Exhaust for [T; N] {
    type Iter = ExhaustArray<T, N>;
    type Factory = [T::Factory; N];
    fn exhaust_factories() -> Self::Iter {
        ExhaustArray {
            state: [(); N].map(|()| peekable_exhaust::<T>()),
            done_zero: false,
        }
    }
    fn from_factory(factory: Self::Factory) -> Self {
        factory.map(T::from_factory)
    }
}

impl<T: Indexable, const N: usize> Indexable for [T; N] {
    const VALUE_COUNT: usize = array_value_count(T::VALUE_COUNT, N);

    fn to_index(array: &Self) -> usize {
        let mut value_index = 0;
        // Same algorithm as assembling digits into a number.
        for element in array {
            value_index *= T::VALUE_COUNT;
            value_index += T::to_index(element);
        }
        value_index
    }

    fn from_index(mut index: usize) -> Self {
        // Note: We could skip `indices` and build the final array in-place, but there
        // is no reverse counterpart to `core::array::from_fn()`, so we'd need one of
        // *  unsafe and `MaybeUninit` to fill the array backwards,
        // * exponentiation to obtain the “digits” in forward order, or
        // * reversing the array of `T` afterward.
        // For now, this is boring and correct and has O(N) space overhead, not O(N*size_of(T)).
        let mut indices: [usize; N] = [0; N];
        for element_index in indices.iter_mut().rev() {
            *element_index = index.rem_euclid(T::VALUE_COUNT);
            index = index.div_euclid(T::VALUE_COUNT);
        }
        indices.map(T::from_index)
    }
}

/// Iterator implementation of `[T; N]::exhaust()`.
pub struct ExhaustArray<T: Exhaust, const N: usize> {
    state: [iter::Peekable<T::Iter>; N],
    done_zero: bool,
}

impl<T, const N: usize> fmt::Debug for ExhaustArray<T, N>
where
    T: Exhaust<Iter: fmt::Debug, Factory: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExhaustArray")
            .field("state", &self.state)
            .field("done_zero", &self.done_zero)
            .finish()
    }
}

impl<T: Exhaust, const N: usize> Clone for ExhaustArray<T, N> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            done_zero: self.done_zero,
        }
    }
}

impl<T: Exhaust, const N: usize> Iterator for ExhaustArray<T, N> {
    type Item = [T::Factory; N];

    fn next(&mut self) -> Option<Self::Item> {
        if N == 0 {
            return if self.done_zero {
                None
            } else {
                self.done_zero = true;
                // This is just `Some([])` in disguise
                Some([(); N].map(|()| unreachable!()))
            };
        }

        // Check if we have a next item
        let has_next = self
            .state
            .iter_mut()
            .all(|value_iter| value_iter.peek().is_some());

        if !has_next {
            return None;
        }

        // Gather that next item.
        // unwrap() cannot fail because we checked with peek().
        let mut i = 0;
        let item = [(); N].map(|()| {
            let element = if i == N - 1 {
                // Advance the "last digit".
                self.state[i].next().unwrap()
            } else {
                // Don't advance the others
                self.state[i].peek().unwrap().clone()
            };
            i += 1;
            element
        });

        // "Carry": if the rightmost iterator is exhausted, advance the one to the left,
        // and repeat for all but the leftmost. If the leftmost is exhausted, we'll stop
        // on the next iteration.
        for i in (1..N).rev() {
            let (high, low) = &mut self.state.split_at_mut(i);
            if !carry(high.last_mut().unwrap(), &mut low[0], peekable_exhaust::<T>) {
                break;
            }
        }

        Some(item)
    }
}

impl<T: Exhaust, const N: usize> iter::FusedIterator for ExhaustArray<T, N> {}

#[mutants::skip] // difficult to test the edge cases exactly unless we expose this fn
const fn array_value_count(inner_value_count: usize, length: usize) -> usize {
    #[allow(clippy::cast_possible_truncation)]
    let n = if length as u128 > u32::MAX as u128 {
        panic!("array length too large for Indexable");
    } else {
        length as u32
    };

    match inner_value_count.checked_pow(n) {
        Some(count) => count,
        // Ideally, we would print the value, but formatting in const expressions is not
        // yet allowed.
        None => panic!("total value count too large for Indexable"),
    }
}
