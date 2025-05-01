use core::iter;

use crate::patterns::{
    delegate_factory_and_iter, factory_is_self, impl_newtype_generic, impl_via_range,
};
use crate::Exhaust;

impl Exhaust for () {
    type Iter = iter::Once<()>;
    fn exhaust_factories() -> Self::Iter {
        iter::once(())
    }
    factory_is_self!();
}

// Implement single-element tuples in the same way we implement other generic containers.
impl_newtype_generic!(T: [], (T,), |x| (x,));

// Generates tuple implementations from 2 to 12 items.
// 12 was chosen as the same size the standard library offers.
exhaust_macros::impl_exhaust_for_tuples!(12);

impl_via_range!(char, '\x00', char::MAX);
impl_via_range!(i8, i8::MIN, i8::MAX);
impl_via_range!(u8, u8::MIN, u8::MAX);
impl_via_range!(i16, i16::MIN, i16::MAX);
impl_via_range!(u16, u16::MIN, u16::MAX);
impl_via_range!(i32, i32::MIN, i32::MAX);
impl_via_range!(u32, u32::MIN, u32::MAX);
// i64 and larger sizes are not implemented because it is not feasible to exhaust them.
/// Note: The floats produced include many `NaN`s (all unequal in representation).
impl Exhaust for f32 {
    delegate_factory_and_iter!(u32);

    fn from_factory(factory: Self::Factory) -> Self {
        f32::from_bits(factory)
    }
}
