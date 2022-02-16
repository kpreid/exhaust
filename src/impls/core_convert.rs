use core::iter;

use crate::Exhaust;

impl Exhaust for core::convert::Infallible {
    type Iter = iter::Empty<core::convert::Infallible>;
    fn exhaust() -> Self::Iter {
        iter::empty()
    }
}
