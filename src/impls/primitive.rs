use crate::patterns::{
    delegate_factory_and_iter, impl_newtype_generic_indexable, impl_singleton, impl_via_range,
};
use crate::{Exhaust, Indexable};

impl_singleton!([], ());

// Implement single-element tuples in the same way we implement other generic containers.
impl_newtype_generic_indexable!(T: [], (T,), |x| (x,), tuple_1_get);
fn tuple_1_get<T>(value: &(T,)) -> &T {
    &value.0
}

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

impl Indexable for u8 {
    const VALUE_COUNT: usize = 1 << Self::BITS as usize;

    fn to_index(value: &Self) -> usize {
        usize::from(*value)
    }

    fn from_index(index: usize) -> Self {
        #![allow(clippy::cast_possible_truncation)]
        Self::try_from(index).unwrap()
    }
}

impl Indexable for i8 {
    const VALUE_COUNT: usize = 1 << Self::BITS as usize;

    fn to_index(value: &Self) -> usize {
        #![allow(clippy::cast_sign_loss)]
        // TODO: When MSRV >= 1.87, express this as usize::from(....cast_unsigned())
        value.wrapping_sub(Self::MIN) as u8 as usize
    }

    fn from_index(index: usize) -> Self {
        #![allow(clippy::cast_possible_truncation)]
        assert!(index < Self::VALUE_COUNT);
        (index as Self).wrapping_add(Self::MIN)
    }
}

// Larger integers cannot implement `Indexable` because their `VALUE_COUNT` cannot be losslessly
// converted to `usize`.
// TODO: Consider adding a pragmatic conditional impl for `i16`/`u16`.
