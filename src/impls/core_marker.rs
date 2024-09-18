use core::marker;

use crate::patterns::impl_singleton;

impl_singleton!([T], marker::PhantomData<T>);
impl_singleton!([], marker::PhantomPinned);
