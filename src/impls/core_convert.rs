use core::iter;

use crate::Exhaust;

impl Exhaust for core::convert::Infallible {
    type Iter = iter::Empty<core::convert::Infallible>;
    fn exhaust_factories() -> Self::Iter {
        iter::empty()
    }
    crate::patterns::factory_is_self!();
}
