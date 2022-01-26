use core::iter;

use super::Exhaust;

impl Exhaust for () {
    type Iter = iter::Once<()>;
    fn exhaust() -> Self::Iter {
        iter::once(())
    }
}

impl Exhaust for bool {
    type Iter = core::array::IntoIter<bool, 2>;
    fn exhaust() -> Self::Iter {
        [false, true].into_iter()
    }
}

impl<T: Exhaust, const N: usize> Exhaust for [T; N] {
    type Iter = ExhaustArray<T, N>;
    fn exhaust() -> Self::Iter {
        ExhaustArray {
            state: [(); N].map(|_| T::exhaust().peekable()),
        }
    }
}

/// Iterator implementation of `[T; N]::exhaust()`.
#[derive(Clone, Debug)]
pub struct ExhaustArray<T: Exhaust, const N: usize> {
    state: [iter::Peekable<T::Iter>; N],
}

impl<T: Exhaust, const N: usize> Iterator for ExhaustArray<T, N> {
    type Item = [T; N];
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
