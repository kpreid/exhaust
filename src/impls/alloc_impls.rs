use core::fmt;
use core::pin::Pin;

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::rc::Rc;

use crate::patterns::impl_newtype_generic;
use crate::Exhaust;

impl_newtype_generic!(T: [], Box<T>, Box::new);
impl_newtype_generic!(T: [], Rc<T>, Rc::new);
impl_newtype_generic!(T: [], Pin<Box<T>>, Box::pin);
impl_newtype_generic!(T: [], Pin<Rc<T>>, Rc::pin);

// Note: This impl is essentially identical to the one for `HashSet`.
impl<T: Exhaust + Ord> Exhaust for BTreeSet<T> {
    type Iter = ExhaustBTreeSet<T>;
    fn exhaust() -> Self::Iter {
        ExhaustBTreeSet(itertools::Itertools::powerset(T::exhaust()))
    }
}

// TODO: Instead of delegating to itertools, we should implement our own powerset
// iterator, to provide the preferred ordering of elements.
#[derive(Clone)]
pub struct ExhaustBTreeSet<T: Exhaust>(itertools::Powerset<<T as Exhaust>::Iter>);

impl<T: Exhaust + Ord> Iterator for ExhaustBTreeSet<T> {
    type Item = BTreeSet<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(BTreeSet::from_iter)
    }
}

impl<T: Exhaust> fmt::Debug for ExhaustBTreeSet<T>
where
    T: fmt::Debug,
    T::Iter: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ExhaustPowerset").field(&self.0).finish()
    }
}
