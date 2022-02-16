//! Macros to generate particular `Exhaust` implementation styles.
//!
//! These are not public; they are intended to be freely modified as suits
//! the use cases that come up internally.

macro_rules! impl_via_array {
    ($self:ty, $array:expr) => {
        impl $crate::Exhaust for $self {
            type Iter = ::core::array::IntoIter<Self, { $array.len() }>;
            fn exhaust() -> Self::Iter {
                $array.into_iter()
            }
        }
    };
}
pub(crate) use impl_via_array;

macro_rules! impl_via_range {
    ($self:ty, $start:expr, $end:expr) => {
        impl $crate::Exhaust for $self {
            type Iter = ::core::ops::RangeInclusive<$self>;
            fn exhaust() -> Self::Iter {
                ($start)..=($end)
            }
        }
    };
}
pub(crate) use impl_via_range;

macro_rules! impl_newtype_generic {
    ($tvar:ident : [ $( $bounds:tt )* ] , $container:ty, $wrap_fn:expr) => {
        impl<$tvar: $crate::Exhaust> $crate::Exhaust for $container
        where
            $tvar: $( $bounds )*
        {
            type Iter =
                ::core::iter::Map<<$tvar as $crate::Exhaust>::Iter, fn($tvar) -> $container>;
            fn exhaust() -> Self::Iter {
                <$tvar as $crate::Exhaust>::exhaust().map($wrap_fn)
            }
        }
    };
}
pub(crate) use impl_newtype_generic;
