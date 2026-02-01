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

/// Implement `Exhaust` for `$self` by iterating over the given array.
/// The array must have no more than `u8::MAX - 1` (254) elements.
macro_rules! impl_via_array {
    ($self:ty, $array:expr) => {
        // block to hide the non-uniquely named items
        const _: () = {
            impl $crate::Exhaust for $self {
                type Iter = ExhaustIter;
                fn exhaust_factories() -> Self::Iter {
                    #![allow(clippy::cast_possible_truncation)]
                    const {
                        assert!(VALUES.len() < u8::MAX as usize);
                        ExhaustIter {
                            index_iter: 0..{ VALUES.len() as u8 },
                        }
                    }
                }
                $crate::patterns::factory_is_self!();
            }

            const VALUES: &[$self; $array.len()] = &$array;

            // Opaque iterator struct for this particular `Exhaust` implementation.
            // Public-in-private type as a substitute for associated type `impl Iterator`.
            #[derive(Clone, Debug)] // TODO: custom Debug
            pub struct ExhaustIter {
                // Indices into `VALUES` to produce.
                index_iter: ::core::ops::Range<u8>,
            }

            // Note: We could easily decide that the factory is a `u8` instead of doing
            // `factory_is_self` style, but we want to offer `factory_is_self` since it
            // is the maximally flexible option.

            impl Iterator for ExhaustIter {
                type Item = $self;
                #[inline]
                fn next(&mut self) -> Option<Self::Item> {
                    match self.index_iter.next() {
                        Some(index) => Some(VALUES[usize::from(index)]),
                        None => None,
                    }
                }

                #[inline]
                fn size_hint(&self) -> (usize, Option<usize>) {
                    self.index_iter.size_hint()
                }
            }
            impl DoubleEndedIterator for ExhaustIter {
                #[inline]
                fn next_back(&mut self) -> Option<Self::Item> {
                    match self.index_iter.next_back() {
                        Some(index) => Some(VALUES[usize::from(index)]),
                        None => None,
                    }
                }
            }
            impl ::core::iter::FusedIterator for ExhaustIter {}
        };
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
