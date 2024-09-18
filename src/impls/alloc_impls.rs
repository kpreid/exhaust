use core::iter::FusedIterator;
use core::pin::Pin;
use core::{fmt, iter};

use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::rc::Rc;
use alloc::vec::Vec;

use itertools::Itertools as _;

use crate::iteration::peekable_exhaust;
use crate::patterns::impl_newtype_generic;
use crate::Exhaust;

impl_newtype_generic!(T: [], Box<T>, Box::new);
impl_newtype_generic!(T: [], Rc<T>, Rc::new);
impl_newtype_generic!(T: [], Pin<Box<T>>, Box::pin);
impl_newtype_generic!(T: [], Pin<Rc<T>>, Rc::pin);

/// Note that this implementation necessarily ignores the borrowed versus owned distinction;
/// every value returned will be a [`Cow::Owned`], not a [`Cow::Borrowed`].
/// This agrees with the [`PartialEq`] implementation for [`Cow`], which considers
/// owned and borrowed to be equal.
impl<'a, T: ?Sized + ToOwned<Owned = O>, O: Exhaust> Exhaust for Cow<'a, T> {
    type Iter = <O as Exhaust>::Iter;
    type Factory = O::Factory;

    fn exhaust_factories() -> Self::Iter {
        O::exhaust_factories()
    }

    fn from_factory(factory: Self::Factory) -> Self {
        Cow::Owned(O::from_factory(factory))
    }
}

// Note: This impl is essentially identical to the one for `HashSet`.
impl<T: Exhaust + Ord> Exhaust for BTreeSet<T> {
    type Iter = ExhaustSet<T>;
    type Factory = Vec<T::Factory>;
    fn exhaust_factories() -> Self::Iter {
        ExhaustSet::default()
    }
    fn from_factory(factory: Self::Factory) -> Self {
        factory.into_iter().map(T::from_factory).collect()
    }
}

// TODO: Instead of delegating to itertools, we should implement our own powerset
// iterator, to provide the preferred ordering of elements.
pub struct ExhaustSet<T: Exhaust>(itertools::Powerset<<T as Exhaust>::Iter>);

impl<T: Exhaust> Default for ExhaustSet<T> {
    fn default() -> Self {
        ExhaustSet(itertools::Itertools::powerset(T::exhaust_factories()))
    }
}

impl<T: Exhaust> Clone for ExhaustSet<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Exhaust> Iterator for ExhaustSet<T> {
    type Item = Vec<T::Factory>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
impl<T: Exhaust> FusedIterator for ExhaustSet<T> {}

impl<T: Exhaust> fmt::Debug for ExhaustSet<T>
where
    T::Factory: fmt::Debug,
    T::Iter: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ExhaustSet").field(&self.0).finish()
    }
}

pub(crate) type MapFactory<K, V> = Vec<(<K as Exhaust>::Factory, <V as Exhaust>::Factory)>;

impl<K: Exhaust + Ord, V: Exhaust> Exhaust for BTreeMap<K, V> {
    type Iter = ExhaustMap<<BTreeSet<K> as Exhaust>::Iter, V>;
    fn exhaust_factories() -> Self::Iter {
        ExhaustMap::new(peekable_exhaust::<BTreeSet<K>>())
    }

    type Factory = MapFactory<K, V>;

    fn from_factory(factory: Self::Factory) -> Self {
        factory
            .into_iter()
            .map(|(k, v)| (K::from_factory(k), V::from_factory(v)))
            .collect()
    }
}

/// Iterator which exhausts map types.
///
/// * `KI` is an iterator of key factory *sets* (as `Vec`s) that the map should contain.
/// * `V` is the type of the map’s values.
pub struct ExhaustMap<KI, V>
where
    KI: Iterator,
    V: Exhaust,
{
    keys: iter::Peekable<KI>,
    vals: iter::Peekable<itertools::MultiProduct<<V as Exhaust>::Iter>>,
}

impl<KF, KI, V> ExhaustMap<KI, V>
where
    KI: Iterator<Item = Vec<KF>>,
    V: Exhaust,
{
    pub fn new(mut keys: iter::Peekable<KI>) -> Self {
        let key_count = keys.peek().map_or(0, Vec::len);
        ExhaustMap {
            keys,
            vals: itertools::repeat_n(V::exhaust_factories(), key_count)
                .multi_cartesian_product()
                .peekable(),
        }
    }
}

impl<KF, KI, V> Iterator for ExhaustMap<KI, V>
where
    KI: Iterator<Item = Vec<KF>>,
    KF: Clone,
    V: Exhaust,
{
    type Item = Vec<(KF, V::Factory)>;
    fn next(&mut self) -> Option<Self::Item> {
        let keys: Vec<KF> = self.keys.peek()?.clone();
        let vals: Vec<V::Factory> = self.vals.next()?;

        if self.vals.peek().is_none() {
            self.keys.next();
            let key_count = self.keys.peek().map_or(0, Vec::len);
            self.vals = itertools::repeat_n(V::exhaust_factories(), key_count)
                .multi_cartesian_product()
                .peekable();
        }

        Some(keys.into_iter().zip_eq(vals).collect())
    }
}
impl<KF, KI, V> FusedIterator for ExhaustMap<KI, V>
where
    KI: Iterator<Item = Vec<KF>>,
    KF: Clone,
    V: Exhaust,
{
}

impl<KI, V> Clone for ExhaustMap<KI, V>
where
    KI: Iterator<Item: Clone> + Clone,
    V: Exhaust,
{
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            vals: self.vals.clone(),
        }
    }
}

impl<KI, V> fmt::Debug for ExhaustMap<KI, V>
where
    KI: fmt::Debug + Iterator<Item: fmt::Debug>,
    V: Exhaust<Iter: fmt::Debug, Factory: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExhaustMap")
            .field("keys", &self.keys)
            .field("vals", &self.vals)
            .finish()
    }
}
