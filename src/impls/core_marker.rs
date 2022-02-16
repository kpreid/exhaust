use core::iter;

use crate::Exhaust;

impl<T: ?Sized> Exhaust for core::marker::PhantomData<T> {
    type Iter = iter::Once<core::marker::PhantomData<T>>;
    fn exhaust() -> Self::Iter {
        iter::once(core::marker::PhantomData)
    }
}

impl Exhaust for core::marker::PhantomPinned {
    type Iter = iter::Once<core::marker::PhantomPinned>;
    fn exhaust() -> Self::Iter {
        iter::once(core::marker::PhantomPinned)
    }
}
