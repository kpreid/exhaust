use crate::patterns::impl_newtype_generic;
use crate::Exhaust;

impl_newtype_generic!(T: [], core::cmp::Reverse<T>, core::cmp::Reverse);

impl Exhaust for core::cmp::Ordering {
    type Iter = core::array::IntoIter<Self, 3>;
    fn exhaust() -> Self::Iter {
        [Self::Less, Self::Equal, Self::Greater].into_iter()
    }
}
