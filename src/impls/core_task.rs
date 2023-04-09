use core::iter;
use core::task;

use crate::Exhaust;

impl<T: Exhaust> Exhaust for task::Poll<T> {
    type Iter = iter::Map<<Option<T> as Exhaust>::Iter, fn(Option<T>) -> task::Poll<T>>;

    fn exhaust() -> Self::Iter {
        Option::<T>::exhaust().map(option_to_poll as _)
    }
}

fn option_to_poll<T>(opt: Option<T>) -> task::Poll<T> {
    match opt {
        None => task::Poll::Pending,
        Some(val) => task::Poll::Ready(val),
    }
}
