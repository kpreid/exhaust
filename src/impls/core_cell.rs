use core::cell;

use crate::patterns::{delegate_factory_and_iter, impl_newtype_generic};
use crate::Exhaust;

impl_newtype_generic!(T: [], cell::Cell<T>, cell::Cell::new);
impl_newtype_generic!(T: [], cell::RefCell<T>, cell::RefCell::new);
impl_newtype_generic!(T: [], cell::UnsafeCell<T>, cell::UnsafeCell::new);

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
