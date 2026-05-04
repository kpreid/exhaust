use core::iter;
use core::num::{self, NonZero};

use crate::patterns::{impl_newtype_generic, impl_via_array};
use crate::Exhaust;

// -------------------------------------------------------------------------------------------------

macro_rules! impl_nonzero {
    ($t:ty) => {
        impl Exhaust for NonZero<$t> {
            type Iter = ExhaustNonZero<$t, NonZero<$t>>;

            fn exhaust_factories() -> Self::Iter {
                // TODO: This `filter_map()` is tidy and generic, but is probably not the optimal
                // implementation for unsigned numbers, since if `next()` is not inlined, it'll
                // need a comparison with zero on each iteration. But I haven’t checked.
                ExhaustNonZero::<$t, NonZero<$t>>(
                    <$t>::exhaust_factories().filter_map(NonZero::new),
                )
            }

            crate::patterns::factory_is_self!();
        }
    };
}

// Implement `Exhaust` for all `NonZero`-able numbers that are no larger than 32 bits.
// This should match <https://doc.rust-lang.org/std/num/trait.ZeroablePrimitive.html>
// (as long as that's the unstable trait backing `NonZero`), except for those that are too large.
impl_nonzero!(i8);
impl_nonzero!(i16);
impl_nonzero!(i32);
impl_nonzero!(u8);
impl_nonzero!(u16);
impl_nonzero!(u32);

/// Iterator implementation for `NonZero::exhaust()`.
// TODO: This should just be a type_alias_impl_trait for FilterMap when that's stable.
// Right now, it's just public-in-private so unnameable that way.
#[derive(Clone, Debug)]
#[doc(hidden)]
#[allow(clippy::type_complexity)]
pub struct ExhaustNonZero<T: Exhaust, N>(
    iter::FilterMap<<T as Exhaust>::Iter, fn(<T as Exhaust>::Factory) -> Option<N>>,
);

impl<T: Exhaust, N> Iterator for ExhaustNonZero<T, N> {
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // This is the size hint from `FilterMap`, which is pessimistic (the lower bound
        // will always be 0).
        let (mut lower, upper) = self.0.size_hint();

        // Fix it using our knowledge that we are filtering out only one item.
        if let Some(upper) = upper {
            lower = upper.saturating_sub(1);
        }

        (lower, upper)
    }
}
impl<T: Exhaust, N> iter::FusedIterator for ExhaustNonZero<T, N> {}

// -------------------------------------------------------------------------------------------------

impl_via_array!(
    num::FpCategory,
    [
        num::FpCategory::Nan,
        num::FpCategory::Infinite,
        num::FpCategory::Zero,
        num::FpCategory::Subnormal,
        num::FpCategory::Normal,
    ]
);

impl_newtype_generic!(T: [], num::Wrapping<T>, num::Wrapping);
