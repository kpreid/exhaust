use core::pin::Pin;

use alloc::boxed::Box;
use alloc::rc::Rc;

use crate::patterns::impl_newtype_generic;

impl_newtype_generic!(T: [], Box<T>, Box::new);
impl_newtype_generic!(T: [], Rc<T>, Rc::new);
impl_newtype_generic!(T: [], Pin<Box<T>>, Box::pin);
impl_newtype_generic!(T: [], Pin<Rc<T>>, Rc::pin);
