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

/// Delegate the `type Iter`, `type Factory`, and `from_factory` to another type.
/// Use this macro inside an `impl Exhaust`, and implement `from_factory()` to convert
/// from `$delegate`’s factory to `Self` (usually using `$delegate::from_factory()`).
macro_rules! delegate_factory_and_iter {
    ($delegate:ty) => {
        type Iter = <$delegate as $crate::Exhaust>::Iter;
        type Factory = <$delegate as $crate::Exhaust>::Factory;

        fn exhaust_factories() -> Self::Iter {
            <$delegate as $crate::Exhaust>::exhaust_factories()
        }
    };
}
pub(crate) use delegate_factory_and_iter;

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

macro_rules! impl_via_small_list {
    ($self:ty, [$($item:path),* $(,)?]) => {
        impl $crate::Exhaust for $self {
            type Iter = ::core::array::IntoIter<Self, { [$($item,)*].len() }>;
            fn exhaust_factories() -> Self::Iter {
                // TODO: This produces a more complex iterator than necessary
                [$($item,)*].into_iter()
            }
            $crate::patterns::factory_is_self!();
        }

        impl $crate::Indexable for $self {
            const VALUE_COUNT: usize = { [$($item,)*].len() };

            fn to_index(value: &Self) -> usize {
                $crate::patterns::match_to_index!(value ; $($item),*)
            }

            fn from_index(index: usize) -> Self {
                (const { &[$($item,)*] }[index])
            }
        }
    };
}
pub(crate) use impl_via_small_list;

// Helper for impl_via_small_list
macro_rules! match_to_index {
    ($var:ident ; $p0:path, $p1:path) => {
        match $var {
            $p0 => 0,
            $p1 => 1,
        }
    };
    ($var:ident ; $p0:path, $p1:path, $p2:path) => {
        match $var {
            $p0 => 0,
            $p1 => 1,
            $p2 => 2,
        }
    };
    ($var:ident ; $p0:path, $p1:path, $p2:path, $p3:path) => {
        match $var {
            $p0 => 0,
            $p1 => 1,
            $p2 => 2,
            $p3 => 3,
        }
    };
    ($var:ident ; $p0:path, $p1:path, $p2:path, $p3:path, $p4:path) => {
        match $var {
            $p0 => 0,
            $p1 => 1,
            $p2 => 2,
            $p3 => 3,
            $p4 => 4,
        }
    };
}
pub(crate) use match_to_index;

macro_rules! impl_via_range {
    ($self:ty, $start:expr, $end:expr) => {
        impl $crate::Exhaust for $self {
            type Iter = ::core::ops::RangeInclusive<$self>;
            fn exhaust_factories() -> Self::Iter {
                $start..=$end
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
