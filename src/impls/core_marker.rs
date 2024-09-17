use core::iter;

use crate::Exhaust;

impl<T: ?Sized> Exhaust for core::marker::PhantomData<T> {
    type Iter = iter::Once<core::marker::PhantomData<T>>;
    fn exhaust_factories() -> Self::Iter {
        iter::once(core::marker::PhantomData)
    }
    crate::patterns::factory_is_self!();
}

impl Exhaust for core::marker::PhantomPinned {
    type Iter = iter::Once<core::marker::PhantomPinned>;
    fn exhaust_factories() -> Self::Iter {
        iter::once(core::marker::PhantomPinned)
    }
    crate::patterns::factory_is_self!();
}
