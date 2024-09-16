use core::pin::Pin;
use core::{fmt, iter};

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::rc::Rc;
use alloc::vec::Vec;

use itertools::Itertools as _;

use crate::iteration::{peekable_exhaust, Pei};
use crate::patterns::impl_newtype_generic;
use crate::Exhaust;

impl_newtype_generic!(T: [], Box<T>, Box::new);
impl_newtype_generic!(T: [], Rc<T>, Rc::new);
impl_newtype_generic!(T: [], Pin<Box<T>>, Box::pin);
impl_newtype_generic!(T: [], Pin<Rc<T>>, Rc::pin);

impl<'a, T: Exhaust> Exhaust for Cow<'a, T> {
    type Iter = ::core::iter::Map<<T as Exhaust>::Iter, fn(T) -> Cow<'a, T>>;

    /// Note that this implementation necessarily ignores the borrowed versus owned distinction;
    /// every value returned will be a [`Cow::Owned`], not a [`Cow::Borrowed`].
    /// This agrees with the [`PartialEq`] implementation for [`Cow`], which considers
    /// owned and borrowed to be equal.
    fn exhaust() -> Self::Iter {
        <T as Exhaust>::exhaust().map(Cow::Owned)
    }
}

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
        f.debug_tuple("ExhaustBTreeSet").field(&self.0).finish()
    }
}

impl<K: Exhaust + Ord, V: Exhaust> Exhaust for BTreeMap<K, V> {
    type Iter = ExhaustBTreeMap<K, V>;
    fn exhaust() -> Self::Iter {
        let mut keys: Pei<BTreeSet<K>> = peekable_exhaust::<BTreeSet<K>>();
        let key_count = keys.peek().map_or(0, BTreeSet::len);
        ExhaustBTreeMap {
            keys,
            vals: itertools::repeat_n(V::exhaust(), key_count)
                .multi_cartesian_product()
                .peekable(),
        }
    }
}

// Note: This iterator is essentially identical to the one for `HashMap`.
//
// TODO: Eliminate the construction of actual BTreeSet keys because it's not beneficial
pub struct ExhaustBTreeMap<K: Exhaust + Ord, V: Exhaust> {
    keys: Pei<BTreeSet<K>>,
    vals: iter::Peekable<itertools::MultiProduct<<V as Exhaust>::Iter>>,
}

impl<K: Exhaust + Ord, V: Exhaust> Iterator for ExhaustBTreeMap<K, V> {
    type Item = BTreeMap<K, V>;
    fn next(&mut self) -> Option<Self::Item> {
        let keys: BTreeSet<K> = self.keys.peek()?.clone();
        let vals: Vec<V> = self.vals.next()?;

        if self.vals.peek().is_none() {
            self.keys.next();
            let key_count = self.keys.peek().map_or(0, BTreeSet::len);
            self.vals = itertools::repeat_n(V::exhaust(), key_count)
                .multi_cartesian_product()
                .peekable();
        }

        Some(keys.into_iter().zip_eq(vals).collect())
    }
}

impl<K, V> Clone for ExhaustBTreeMap<K, V>
where
    K: Exhaust + Ord,
    V: Exhaust,
{
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            vals: self.vals.clone(),
        }
    }
}

#[allow(clippy::type_repetition_in_bounds)] // TODO: report false positive
impl<K, V> fmt::Debug for ExhaustBTreeMap<K, V>
where
    K: fmt::Debug + Exhaust + Ord,
    V: fmt::Debug + Exhaust,
    K::Iter: fmt::Debug,
    V::Iter: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExhaustBTreeMap")
            .field("keys", &self.keys)
            .field("vals", &self.vals)
            .finish()
    }
}
