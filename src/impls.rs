use core::iter;

use super::Exhaust;

macro_rules! impl_newtype_generic {
    ($tvar:ident, $container:ty, $wrap_fn:expr) => {
        impl<$tvar: $crate::Exhaust> $crate::Exhaust for $container {
            type Iter =
                ::core::iter::Map<<$tvar as $crate::Exhaust>::Iter, fn($tvar) -> $container>;
            fn exhaust() -> Self::Iter {
                <$tvar as $crate::Exhaust>::exhaust().map($wrap_fn)
            }
        }
    };
}

impl Exhaust for () {
    type Iter = iter::Once<()>;
    fn exhaust() -> Self::Iter {
        iter::once(())
    }
}

impl Exhaust for bool {
    type Iter = core::array::IntoIter<bool, 2>;
    fn exhaust() -> Self::Iter {
        [false, true].into_iter()
    }
}

impl<T: Exhaust, const N: usize> Exhaust for [T; N] {
    type Iter = ExhaustArray<T, N>;
    fn exhaust() -> Self::Iter {
        ExhaustArray {
            state: [(); N].map(|_| T::exhaust().peekable()),
        }
    }
}

/// Iterator implementation of `[T; N]::exhaust()`.
#[derive(Clone, Debug)]
pub struct ExhaustArray<T: Exhaust, const N: usize> {
    state: [iter::Peekable<T::Iter>; N],
}

impl<T: Exhaust, const N: usize> Iterator for ExhaustArray<T, N> {
    type Item = [T; N];
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[cfg(feature = "alloc")]
mod alloc_impls {
    use alloc::boxed::Box;
    use alloc::rc::Rc;

    impl_newtype_generic!(T, Box<T>, Box::new);
    impl_newtype_generic!(T, Rc<T>, Rc::new);
}

#[cfg(feature = "std")]
mod std_impls {
    use std::sync::Arc;

    impl_newtype_generic!(T, Arc<T>, Arc::new);
}
