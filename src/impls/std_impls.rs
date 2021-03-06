use core::fmt;
use core::hash::{BuildHasher, Hash};
use core::marker::PhantomData;
use core::pin::Pin;

use std::collections::HashSet;
use std::sync;
use std::vec::Vec;

use crate::patterns::impl_newtype_generic;
use crate::Exhaust;

// Note: This impl is essentially identical to the one for `BTreeSet`.
impl<T, S> Exhaust for HashSet<T, S>
where
    T: Exhaust + Eq + Hash,
    S: Clone + Default + BuildHasher,
{
    type Iter = ExhaustHashSet<T, S>;
    fn exhaust() -> Self::Iter {
        ExhaustHashSet {
            iter: itertools::Itertools::powerset(T::exhaust()),
            _phantom: PhantomData,
        }
    }
}

// TODO: Instead of delegating to itertools, we could implement our own powerset iterator,
// which we will eventually want to do for the sake of BTreeSet ordering anyway.
pub struct ExhaustHashSet<T: Exhaust, S> {
    iter: itertools::Powerset<<T as Exhaust>::Iter>,
    _phantom: PhantomData<fn() -> S>,
}

impl<T, S> Iterator for ExhaustHashSet<T, S>
where
    T: Exhaust + Eq + Hash,
    S: Default + BuildHasher,
{
    type Item = HashSet<T, S>;
    fn next(&mut self) -> Option<Self::Item> {
        let items: Vec<T> = self.iter.next()?;
        let mut set = HashSet::with_capacity_and_hasher(items.len(), S::default());
        set.extend(items);
        Some(set)
    }
}

impl<T: Exhaust, S> Clone for ExhaustHashSet<T, S> {
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Exhaust, S> fmt::Debug for ExhaustHashSet<T, S>
where
    T: fmt::Debug,
    T::Iter: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ExhaustPowerset").field(&self.iter).finish()
    }
}

impl_newtype_generic!(T: [], sync::Arc<T>, sync::Arc::new);
impl_newtype_generic!(T: [], Pin<sync::Arc<T>>, sync::Arc::pin);

// Mutex and RwLock are not Clone. This is evidence that we shouldn't have a Clone bound.
// impl_newtype_generic!(T: [], sync::Mutex<T>, sync::Mutex::new);
// impl_newtype_generic!(T: [], sync::RwLock<T>, sync::RwLock::new);

// Cannot implement Exhaust for sync::Once because it is not Clone.
