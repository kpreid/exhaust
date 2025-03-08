use core::task;

use crate::patterns::delegate_factory_and_iter;
use crate::Exhaust;

impl<T: Exhaust> Exhaust for task::Poll<T> {
    delegate_factory_and_iter!(Option<T>);

    fn from_factory(factory: Self::Factory) -> Self {
        match Option::<T>::from_factory(factory) {
            None => task::Poll::Pending,
            Some(val) => task::Poll::Ready(val),
        }
    }
}
