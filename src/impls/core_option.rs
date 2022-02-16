use core::iter;

use crate::Exhaust;

impl<T: Exhaust> Exhaust for Option<T> {
    #![allow(clippy::type_complexity)] // TODO: use macro to generate an opaque iter
    type Iter =
        iter::Chain<iter::Once<Option<T>>, iter::Map<<T as Exhaust>::Iter, fn(T) -> Option<T>>>;

    fn exhaust() -> Self::Iter {
        iter::once(None).chain(T::exhaust().map(Some as _))
    }
}
