use core::cell;

use crate::patterns::{delegate_factory_and_iter, impl_newtype_generic};
use crate::{Exhaust, Indexable};

impl_newtype_generic!(T: [], cell::Cell<T>, cell::Cell::new);
impl_newtype_generic!(T: [], cell::RefCell<T>, cell::RefCell::new);
impl_newtype_generic!(T: [], cell::UnsafeCell<T>, cell::UnsafeCell::new);

impl<T: Copy + Indexable> Indexable for cell::Cell<T> {
    const VALUE_COUNT: usize = T::VALUE_COUNT;

    fn to_index(value: &Self) -> usize {
        T::to_index(&value.get())
    }

    fn from_index(index: usize) -> Self {
        cell::Cell::new(T::from_index(index))
    }
}

impl<T: Exhaust> Exhaust for cell::OnceCell<T> {
    delegate_factory_and_iter!(Option<T>);

    fn from_factory(factory: Self::Factory) -> Self {
        let cell = cell::OnceCell::new();
        if let Some(value) = Option::<T>::from_factory(factory) {
            match cell.set(value) {
                Ok(()) => {}
                Err(_) => unreachable!(),
            }
        }
        cell
    }
}
