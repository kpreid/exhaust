//! Macros to generate particular `Exhaust` implementation styles.
//!
//! These are not public; they are intended to be freely modified as suits
//! the use cases that come up internally.

macro_rules! factory_is_self {
    () => {
        type Factory = Self;

        fn from_factory(factory: Self::Factory) -> Self {
            factory
        }
    };
}
pub(crate) use factory_is_self;

/// Implementation for types with exactly one value.
macro_rules! impl_singleton {
    // if Default is implemented
    ([$($generics:tt)*], $self:ty) => {
        impl<$($generics)*> $crate::Exhaust for $self {
            type Iter = ::core::iter::Once<()>;
            type Factory = ();
            fn exhaust_factories() -> Self::Iter {
                ::core::iter::once(())
            }
            fn from_factory((): Self::Factory) -> Self {
                ::core::default::Default::default()
            }
        }
    };
    // if Default is not implemented
    ([$($generics:tt)*], $self:ty, $ctor:expr) => {
        impl<$($generics)*> $crate::Exhaust for $self {
            type Iter = ::core::iter::Once<()>;
            type Factory = ();
            fn exhaust_factories() -> Self::Iter {
                ::core::iter::once(())
            }
            fn from_factory((): Self::Factory) -> Self {
                $ctor
            }
        }
    };
}
pub(crate) use impl_singleton;

macro_rules! impl_via_array {
    ($self:ty, $array:expr) => {
        impl $crate::Exhaust for $self {
            type Iter = ::core::array::IntoIter<Self, { $array.len() }>;
            fn exhaust_factories() -> Self::Iter {
                $array.into_iter()
            }
            $crate::patterns::factory_is_self!();
        }
    };
}
pub(crate) use impl_via_array;

macro_rules! impl_via_range {
    ($self:ty, $start:expr, $end:expr) => {
        impl $crate::Exhaust for $self {
            type Iter = ::core::ops::RangeInclusive<$self>;
            fn exhaust_factories() -> Self::Iter {
                (($start)..=($end))
            }
            $crate::patterns::factory_is_self!();
        }
    };
}
pub(crate) use impl_via_range;

/// Implement [`Exhaust`] for a 'newtype' that has one generic field that must also implement
/// [`Exhaust`].
///
/// As an easier to implement syntax, generic bounds must be written inside of square brackets:
/// `impl_newtype_generic!(T: [Foo], Bar, Bar::new)`
macro_rules! impl_newtype_generic {
    ($tvar:ident : [ $( $bounds:tt )* ] , $container:ty, $wrap_fn:expr) => {
        impl<$tvar: $crate::Exhaust> $crate::Exhaust for $container
        where
            $tvar: $( $bounds )*
        {
            type Iter = <$tvar as $crate::Exhaust>::Iter;
            fn exhaust_factories() -> Self::Iter {
                <$tvar as $crate::Exhaust>::exhaust_factories()
            }

            type Factory = <$tvar as $crate::Exhaust>::Factory;
            fn from_factory(factory: Self::Factory) -> Self {
                $wrap_fn(<$tvar as $crate::Exhaust>::from_factory(factory))
            }
        }
    };
}
pub(crate) use impl_newtype_generic;
