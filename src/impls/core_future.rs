use core::{future, iter};

use crate::patterns::impl_newtype_generic;
use crate::Exhaust;

impl<T> Exhaust for future::Pending<T> {
    type Iter = iter::Once<future::Pending<T>>;
    fn exhaust_factories() -> Self::Iter {
        iter::once(future::pending())
    }
    crate::patterns::factory_is_self!();
}
impl_newtype_generic!(T: [], future::Ready<T>, future::ready);
