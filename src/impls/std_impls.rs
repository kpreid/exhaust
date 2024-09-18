use core::hash::{BuildHasher, Hash};
use core::iter;
use core::pin::Pin;

use std::collections::{HashMap, HashSet};
use std::sync;
use std::vec::Vec;

use crate::iteration::{peekable_exhaust, FlatZipMap};
use crate::patterns::{factory_is_self, impl_newtype_generic, impl_singleton};
use crate::Exhaust;

use super::alloc_impls::{ExhaustMap, ExhaustSet, MapFactory};

// Note: This impl is essentially identical to the one for `BTreeSet`.
impl<T, S> Exhaust for HashSet<T, S>
where
    T: Exhaust + Eq + Hash,
    S: Clone + Default + BuildHasher,
{
    type Iter = ExhaustSet<T>;
    type Factory = Vec<T::Factory>;
    fn exhaust_factories() -> Self::Iter {
        ExhaustSet::default()
    }

    fn from_factory(factory: Self::Factory) -> Self {
        factory.into_iter().map(T::from_factory).collect()
    }
}

/// **Caution:** The order in which this iterator produces elements is currently
/// nondeterministic if the hasher `S` is.
/// (This might be improved in the future.)
// TODO: I think the above note is obsolete.
impl<K, V, S> Exhaust for HashMap<K, V, S>
where
    K: Exhaust + Eq + Hash,
    V: Exhaust,
    S: Clone + Default + BuildHasher,
{
    type Iter = ExhaustMap<<HashSet<K, S> as Exhaust>::Iter, V>;

    fn exhaust_factories() -> Self::Iter {
        ExhaustMap::new(peekable_exhaust::<HashSet<K, S>>())
    }

    type Factory = MapFactory<K, V>;

    fn from_factory(factory: Self::Factory) -> Self {
        factory
            .into_iter()
            .map(|(k, v)| (K::from_factory(k), V::from_factory(v)))
            .collect()
    }
}

impl<T: Exhaust + AsRef<[u8]> + Clone> Exhaust for std::io::Cursor<T> {
    type Iter = FlatZipMap<crate::Produce<T>, std::ops::RangeInclusive<u64>, std::io::Cursor<T>>;
    /// Returns each combination of a buffer state and a cursor position, except for those
    /// where the position is beyond the end of the buffer.
    fn exhaust_factories() -> Self::Iter {
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
    factory_is_self!();
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
    fn exhaust_factories() -> Self::Iter {
        iter::once(std::io::empty())
    }
    factory_is_self!();
}

// TODO: implement this after we no longer have a mandatory `Clone` bound for items
// impl Exhaust for std::io::Repeat {
//     type Iter = iter::Map<<u8 as Exhaust>::Iter, fn(u8) -> std::io::Repeat>;
//     fn exhaust() -> Self::Iter {
//         todo!()
//     }
// }

impl_singleton!([], std::io::Sink);

impl_newtype_generic!(T: [], sync::Arc<T>, sync::Arc::new);
impl_newtype_generic!(T: [], Pin<sync::Arc<T>>, sync::Arc::pin);

// Mutex and RwLock are not Clone. This is evidence that we shouldn't have a Clone bound.
// impl_newtype_generic!(T: [], sync::Mutex<T>, sync::Mutex::new);
// impl_newtype_generic!(T: [], sync::RwLock<T>, sync::RwLock::new);

// Cannot implement Exhaust for sync::Once because it is not Clone.
