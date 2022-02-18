use core::iter;
use core::num;

use paste::paste;

use crate::patterns::{impl_newtype_generic, impl_via_array};
use crate::Exhaust;

macro_rules! impl_nonzero {
    ($t:ty, $nzt:ty) => {
        paste! {
            impl Exhaust for num::$nzt {
                type Iter = [< Exhaust $nzt >];

                fn exhaust() -> Self::Iter {
                    [< Exhaust $nzt >] ($t::exhaust().filter_map(num::$nzt::new))
                }
            }

            #[doc = concat!("Iterator implementation of `", stringify!($nzt), "::exhaust()`.")]
            // TODO: This should just be a type_alias_impl_trait for FilterMap when that's stable.
            #[derive(Clone, Debug)]
            pub struct [< Exhaust $nzt >](iter::FilterMap<<$t as Exhaust>::Iter, fn($t) -> Option<num::$nzt>>);

            impl Iterator for [< Exhaust $nzt >] {
                type Item = num::$nzt;

                fn next(&mut self) -> Option<Self::Item> {
                    self.0.next()
                }
            }
        }
    }
}

impl_nonzero!(i8, NonZeroI8);
impl_nonzero!(i16, NonZeroI16);
impl_nonzero!(i32, NonZeroI32);
impl_nonzero!(u8, NonZeroU8);
impl_nonzero!(u16, NonZeroU16);
impl_nonzero!(u32, NonZeroU32);

impl_via_array!(
    num::FpCategory,
    [
        Self::Nan,
        Self::Infinite,
        Self::Zero,
        Self::Subnormal,
        Self::Normal,
    ]
);

impl_newtype_generic!(T: [], num::Wrapping<T>, num::Wrapping);
