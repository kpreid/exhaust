use core::iter::FusedIterator;
use core::pin::Pin;
use core::{fmt, iter};

use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::rc::Rc;
use alloc::vec::Vec;

use itertools::Itertools as _;

use crate::iteration::{peekable_exhaust, Pei};
use crate::patterns::{delegate_factory_and_iter, impl_newtype_generic};
use crate::Exhaust;

// -------------------------------------------------------------------------------------------------

impl_newtype_generic!(T: [], Box<T>, Box::new);
impl_newtype_generic!(T: [], Rc<T>, Rc::new);
impl_newtype_generic!(T: [], Pin<Box<T>>, Box::pin);
impl_newtype_generic!(T: [], Pin<Rc<T>>, Rc::pin);

/// Note that this implementation necessarily ignores the borrowed versus owned distinction;
/// every value returned will be a [`Cow::Owned`], not a [`Cow::Borrowed`].
/// This agrees with the [`PartialEq`] implementation for [`Cow`], which considers
/// owned and borrowed to be equal.
impl<T: ?Sized + ToOwned<Owned = O>, O: Exhaust> Exhaust for Cow<'_, T> {
    delegate_factory_and_iter!(O);

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

// -------------------------------------------------------------------------------------------------

/// Iterator which exhausts set types, in sorted order if `T::exhaust()` is sorted.
pub struct ExhaustSet<T: Exhaust> {
    /// The next elements of these iterators are all but one of the elements of the next set to be
    /// produced (unless `self.last` is exhausted, in which case we will start backtracking).
    ///
    /// Exception: In the case of the empty set, this is empty, since it can't have -1 elements
    /// as the above rule would otherwise imply.
    list: Vec<Pei<T>>,

    /// If `None`, we should produce the empty set (as the first element).
    /// Otherwise, contains an iterator whose next item is the greatest element of `T` that the
    /// next set produced whose other elements are in `list` will contain.
    ///
    /// This iterator is exhausted when we are backtracking (in which case `self.list` is
    /// non-empty) or when we have produced every set (in which case `self.list` is empty).
    last: Option<Pei<T>>,
}

impl<T: Exhaust> Default for ExhaustSet<T> {
    fn default() -> Self {
        ExhaustSet {
            list: Vec::new(),
            last: None,
        }
    }
}

impl<T: Exhaust> Clone for ExhaustSet<T> {
    fn clone(&self) -> Self {
        Self {
            list: self.list.clone(),
            last: self.last.clone(),
        }
    }
}

impl<T: Exhaust> Iterator for ExhaustSet<T> {
    type Item = Vec<T::Factory>;
    fn next(&mut self) -> Option<Self::Item> {
        let Some(ref mut last_iter) = self.last else {
            // Initial state; time to produce the set with 0 elements, and
            // set up to produce a set with 1 element after this one.
            self.last = Some(peekable_exhaust::<T>());
            return Some(Vec::new());
        };

        loop {
            return if let Some(elem) = last_iter.peek() {
                let item = Vec::from_iter(
                    self.list
                        .iter_mut()
                        .map(|iter| {
                            iter.peek()
                                .expect("no peekable should be exhausted")
                                .clone()
                        })
                        .chain(iter::once(elem.clone())),
                );

                // Start iterating over sets that are 1 element longer than this one,
                // if there are any.
                self.list.push(last_iter.clone());

                // Now that we’ve grabbed the clone we need for self.list, actually advance
                // the iterator in self.last (we only peeked before).
                last_iter.next();

                Some(item)
            } else {
                // self.last is exhausted.
                // This means there are no more sets of length self.list.len() + 1
                // with the prefix currently stored in self.list.
                // Advance the iterator in self.list and make it the new self.last.
                if let Some(mut new_last) = self.list.pop() {
                    _ = new_last.next();
                    *last_iter = new_last;
                    // TODO: can we express this algorithm without a loop?
                    continue;
                } else {
                    // No more sets at all.
                    // Keep the exhausted iterator as a marker to stop (so we're fused).
                    None
                }
            };
        }
    }
}
impl<T: Exhaust> FusedIterator for ExhaustSet<T> {}

impl<T: Exhaust> fmt::Debug for ExhaustSet<T>
where
    T::Factory: fmt::Debug,
    T::Iter: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { list, last } = self;
        f.debug_struct("ExhaustSet")
            .field("list", &list)
            .field("last", &last)
            .finish()
    }
}

// -------------------------------------------------------------------------------------------------

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
