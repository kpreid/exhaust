use core::task;

use crate::Exhaust;

impl<T: Exhaust> Exhaust for task::Poll<T> {
    type Iter = <Option<T> as Exhaust>::Iter;
    type Factory = <Option<T> as Exhaust>::Factory;

    fn exhaust_factories() -> Self::Iter {
        Option::<T>::exhaust_factories()
    }

    fn from_factory(factory: Self::Factory) -> Self {
        match Option::<T>::from_factory(factory) {
            None => task::Poll::Pending,
            Some(val) => task::Poll::Ready(val),
        }
    }
}
