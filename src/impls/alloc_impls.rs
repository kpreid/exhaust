use alloc::boxed::Box;
use alloc::rc::Rc;

use crate::patterns::impl_newtype_generic;

impl_newtype_generic!(T: [], Box<T>, Box::new);
impl_newtype_generic!(T: [], Rc<T>, Rc::new);
