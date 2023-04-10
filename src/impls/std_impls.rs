use core::hash::{BuildHasher, Hash};
use core::marker::PhantomData;
use core::pin::Pin;
use core::{fmt, iter};

use std::collections::{HashMap, HashSet};
use std::sync;
use std::vec::Vec;

use itertools::Itertools as _;

use crate::iteration::{peekable_exhaust, FlatZipMap, Pei};
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

impl<K, V, S> Exhaust for HashMap<K, V, S>
where
    K: Exhaust + Eq + Hash,
    V: Exhaust,
    S: Clone + Default + BuildHasher,
{
    type Iter = ExhaustHashMap<K, V, S>;

    /// **Caution:** The order in which this iterator produces elements is currently
    /// nondeterministic if the hasher `S` is.
    /// (This might be improved in the future.)
    fn exhaust() -> Self::Iter {
        let mut keys: Pei<HashSet<K, S>> = peekable_exhaust::<HashSet<K, S>>();
        let key_count = keys.peek().map_or(0, HashSet::len);
        ExhaustHashMap {
            keys,
            vals: itertools::repeat_n(V::exhaust(), key_count)
                .multi_cartesian_product()
                .peekable(),
        }
    }
}

// Note: This iterator is essentially identical to the one for `BTreeMap`.
//
// TODO: Eliminate the construction of actual HashSet keys because it's not beneficial
pub struct ExhaustHashMap<K, V, S>
where
    K: Exhaust + Eq + Hash,
    V: Exhaust,
    S: Clone + Default + BuildHasher,
{
    keys: Pei<HashSet<K, S>>,
    vals: iter::Peekable<itertools::MultiProduct<<V as Exhaust>::Iter>>,
}

impl<K, V, S> Iterator for ExhaustHashMap<K, V, S>
where
    K: Exhaust + Eq + Hash,
    V: Exhaust,
    S: Clone + Default + BuildHasher,
{
    type Item = HashMap<K, V, S>;
    fn next(&mut self) -> Option<Self::Item> {
        let keys: HashSet<K, S> = self.keys.peek()?.clone();
        let vals: Vec<V> = if keys.is_empty() {
            // Empty sets have no keys and therefore no value iterator elements
            Vec::new()
        } else {
            self.vals.next()?
        };

        if self.vals.peek().is_none() {
            self.keys.next();
            let key_count = self.keys.peek().map_or(0, HashSet::len);
            self.vals = itertools::repeat_n(V::exhaust(), key_count)
                .multi_cartesian_product()
                .peekable();
        }

        Some(keys.into_iter().zip_eq(vals).collect())
    }
}

impl<K, V, S> Clone for ExhaustHashMap<K, V, S>
where
    K: Exhaust + Eq + Hash,
    V: Exhaust,
    S: Clone + Default + BuildHasher,
{
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            vals: self.vals.clone(),
        }
    }
}

impl<K, V, S> fmt::Debug for ExhaustHashMap<K, V, S>
where
    K: fmt::Debug + Exhaust + Eq + Hash,
    V: fmt::Debug + Exhaust,
    K::Iter: fmt::Debug,
    V::Iter: fmt::Debug,
    S: Clone + Default + BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExhaustHashMap")
            .field("keys", &self.keys)
            .field("vals", &self.vals)
            .finish()
    }
}

impl<T: Exhaust + AsRef<[u8]>> Exhaust for std::io::Cursor<T> {
    type Iter = FlatZipMap<<T as Exhaust>::Iter, std::ops::RangeInclusive<u64>, std::io::Cursor<T>>;
    /// Returns each combination of a buffer state and a cursor position, except for those
    /// where the position is beyond the end of the buffer.
    fn exhaust() -> Self::Iter {
        FlatZipMap::new(
            T::exhaust(),
            |buf| 0..=(buf.as_ref().len() as u64),
            |buf, pos| {
                let mut cursor = std::io::Cursor::new(buf);
                cursor.set_position(pos);
                cursor
            },
        )
    }
}

// TODO: implement this after we no longer have a mandatory `Clone` bound for items
// impl<T: io::Read + Exhaust> Exhaust for io::BufReader<T> {
//     type Iter = iter::Map<<T as Exhaust>::Iter, fn(T) -> io::BufReader<T>>;
//
//     fn exhaust() -> Self::Iter {
//         T::exhaust().map(io::BufReader::new)
//     }
// }
//
// impl<T: io::Write + Exhaust> Exhaust for io::BufWriter<T> {
//     type Iter = iter::Map<<T as Exhaust>::Iter, fn(T) -> io::BufWriter<T>>;
//
//     fn exhaust() -> Self::Iter {
//         T::exhaust().map(io::BufWriter::new)
//     }
// }

impl Exhaust for std::io::Empty {
    type Iter = iter::Once<std::io::Empty>;
    fn exhaust() -> Self::Iter {
        iter::once(std::io::empty())
    }
}

// TODO: implement this after we no longer have a mandatory `Clone` bound for items
// impl Exhaust for std::io::Repeat {
//     type Iter = iter::Map<<u8 as Exhaust>::Iter, fn(u8) -> std::io::Repeat>;
//     fn exhaust() -> Self::Iter {
//         todo!()
//     }
// }

impl Exhaust for std::io::Sink {
    type Iter = iter::Once<std::io::Sink>;
    fn exhaust() -> Self::Iter {
        iter::once(std::io::sink())
    }
}

impl_newtype_generic!(T: [], sync::Arc<T>, sync::Arc::new);
impl_newtype_generic!(T: [], Pin<sync::Arc<T>>, sync::Arc::pin);

// Mutex and RwLock are not Clone. This is evidence that we shouldn't have a Clone bound.
// impl_newtype_generic!(T: [], sync::Mutex<T>, sync::Mutex::new);
// impl_newtype_generic!(T: [], sync::RwLock<T>, sync::RwLock::new);

// Cannot implement Exhaust for sync::Once because it is not Clone.
