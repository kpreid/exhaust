use core::iter;

use crate::iteration::{carry, peekable_exhaust};
use crate::patterns::{impl_via_array, impl_via_range};
use crate::Exhaust;

impl Exhaust for () {
    type Iter = iter::Once<()>;
    fn exhaust() -> Self::Iter {
        iter::once(())
    }
}

impl_via_array!(bool, [false, true]);

impl_via_range!(char, '\x00', char::MAX);
impl_via_range!(i8, i8::MIN, i8::MAX);
impl_via_range!(u8, u8::MIN, u8::MAX);
impl_via_range!(i16, i16::MIN, i16::MAX);
impl_via_range!(u16, u16::MIN, u16::MAX);
impl_via_range!(i32, i32::MIN, i32::MAX);
impl_via_range!(u32, u32::MIN, u32::MAX);
// i64 and larger sizes are not implemented because it is not feasible to exhaust them.
/// Note: The floats produced include many `NaN`s (all unequal in representation).
impl Exhaust for f32 {
    type Iter = core::iter::Map<<u32 as Exhaust>::Iter, fn(u32) -> f32>;
    fn exhaust() -> Self::Iter {
        u32::exhaust().map(f32::from_bits)
    }
}

impl<T: Exhaust, const N: usize> Exhaust for [T; N] {
    type Iter = ExhaustArray<T, N>;
    fn exhaust() -> Self::Iter {
        ExhaustArray {
            state: [(); N].map(|_| peekable_exhaust::<T>()),
            done_zero: false,
        }
    }
}

/// Iterator implementation of `[T; N]::exhaust()`.
#[derive(Clone, Debug)]
pub struct ExhaustArray<T: Exhaust, const N: usize> {
    state: [iter::Peekable<T::Iter>; N],
    done_zero: bool,
}

impl<T: Exhaust, const N: usize> Iterator for ExhaustArray<T, N> {
    type Item = [T; N];
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
