use core::iter;
use core::num::{self, NonZero};
use core::ops::RangeInclusive;

use crate::patterns::{
    factory_is_self, impl_iterator_for_newtype, impl_newtype_generic_indexable, impl_via_array,
};
use crate::Exhaust;

// -------------------------------------------------------------------------------------------------

macro_rules! impl_nonzero_signed {
    ($t:ty) => {
        impl Exhaust for NonZero<$t> {
            // TODO: Use a better iterator implementation that does not need to check each element
            // for being zero.
            type Iter = ExhaustNonZeroSigned<$t, NonZero<$t>>;

            fn exhaust_factories() -> Self::Iter {
                ExhaustNonZeroSigned::<$t, NonZero<$t>>(
                    <$t>::exhaust_factories().filter_map(NonZero::new),
                )
            }

            factory_is_self!();
        }
    };
}

macro_rules! impl_nonzero_unsigned {
    ($t:ty) => {
        impl Exhaust for NonZero<$t> {
            // TODO: Once MSRV ≥ Rust 1.96, replace this with
            // type Iter = core::range::RangeInclusiveIter<NonZero<$t>>;
            type Iter = ExhaustNonZeroUnsigned<$t>;

            fn exhaust_factories() -> Self::Iter {
                const { ExhaustNonZeroUnsigned(1..=<$t>::MAX) }
            }

            factory_is_self!();
        }

        // This impl can’t be generic because we can’t name the T: ZeroablePrimitive bound
        // which NonZero<T> requires.
        const _: () = {
            fn nonzero_or_panic(value: $t) -> NonZero<$t> {
                match NonZero::try_from(value) {
                    Ok(nz) => nz,
                    Err(_) => unreachable!(),
                }
            }

            impl_iterator_for_newtype!([] for ExhaustNonZeroUnsigned<$t> {
                type Item = NonZero<$t>;
                fn mapper = nonzero_or_panic;
                double_ended_where [];
            });
            impl iter::FusedIterator for ExhaustNonZeroUnsigned<$t> {}
            impl iter::ExactSizeIterator for ExhaustNonZeroUnsigned<$t> {}
        };
    };
}

// Implement `Exhaust` for all `NonZero`-able numbers that are no larger than 32 bits.
// This should match <https://doc.rust-lang.org/std/num/trait.ZeroablePrimitive.html>
// (as long as that's the unstable trait backing `NonZero`), except for those that are too large.
impl_nonzero_signed!(i8);
impl_nonzero_signed!(i16);
impl_nonzero_signed!(i32);
impl_nonzero_unsigned!(u8);
impl_nonzero_unsigned!(u16);
impl_nonzero_unsigned!(u32);

/// Iterator implementation for `NonZero::exhaust()` on signed integers.
//---
// TODO: This should just be a type_alias_impl_trait for FilterMap when that's stable.
// Right now, it's just public-in-private so unnameable that way.
#[derive(Clone, Debug)]
#[doc(hidden)]
#[allow(clippy::type_complexity)]
pub struct ExhaustNonZeroSigned<T: Exhaust, N>(
    iter::FilterMap<<T as Exhaust>::Iter, fn(<T as Exhaust>::Factory) -> Option<N>>,
);

impl<T: Exhaust, N> Iterator for ExhaustNonZeroSigned<T, N> {
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
impl<T: Exhaust, N> iter::FusedIterator for ExhaustNonZeroSigned<T, N> {}

/// Iterator implementation for `NonZero::exhaust()` on unsigned integers.
//---
// Note: the `Iterator` implementations are in the `impl_nonzero_unsigned!` macro.
//
// TODO: Once MSRV ≥ Rust 1.96, replace this entirely with RangeInclusiveIter<NonZero<$t>>.
#[derive(Clone, Debug)]
#[doc(hidden)]
pub struct ExhaustNonZeroUnsigned<T>(RangeInclusive<T>);

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

impl_newtype_generic_indexable!(T: [], num::Wrapping<T>, num::Wrapping, wrap_get);
fn wrap_get<T>(value: &core::num::Wrapping<T>) -> &T {
    &value.0
}
