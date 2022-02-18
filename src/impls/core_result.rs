use core::{fmt, iter};

use crate::Exhaust;

impl<T: Exhaust, E: Exhaust> Exhaust for Result<T, E> {
    type Iter = ExhaustResult<T, E>;

    fn exhaust() -> Self::Iter {
        ExhaustResult(T::exhaust().map(Ok as _).chain(E::exhaust().map(Err as _)))
    }
}

/// Iterator implementation for `Result::exhaust()`.
#[derive(Clone)]
#[allow(clippy::type_complexity)] // TODO: type_alias_impl_trait
pub struct ExhaustResult<T, E>(
    iter::Chain<
        iter::Map<<T as Exhaust>::Iter, fn(T) -> Result<T, E>>,
        iter::Map<<E as Exhaust>::Iter, fn(E) -> Result<T, E>>,
    >,
)
where
    T: Exhaust,
    E: Exhaust;

impl<T: Exhaust, E: Exhaust> Iterator for ExhaustResult<T, E> {
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[allow(clippy::type_repetition_in_bounds)] // TODO: report false positive
impl<T: Exhaust, E: Exhaust> fmt::Debug for ExhaustResult<T, E>
where
    T::Iter: fmt::Debug,
    E::Iter: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ExhaustResult").field(&self.0).finish()
    }
}
