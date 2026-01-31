#![allow(clippy::wildcard_imports)]

use core::ops::Deref;
use core::pin::Pin;
use core::{fmt, iter};

use crate::iteration::{peekable_exhaust, FlatZipMap};
use crate::patterns::{
    delegate_factory_and_iter, factory_is_self, impl_newtype_generic,
    impl_newtype_generic_indexable, impl_singleton, impl_via_small_list,
};
use crate::Exhaust;

use super::alloc_impls::{ExhaustMap, ExhaustSet, MapFactory};

mod collections {
    use super::*;
    use alloc::vec::Vec;
    use core::hash::{BuildHasher, Hash};
    use std::collections::{HashMap, HashSet};

    // Note: This impl is essentially identical to the one for `BTreeSet`.
    impl<T, S> Exhaust for HashSet<T, S>
    where
        T: Exhaust + Eq + Hash,
        S: Default + BuildHasher,
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

    impl<K, V, S> Exhaust for HashMap<K, V, S>
    where
        K: Exhaust + Eq + Hash,
        V: Exhaust,
        S: Default + BuildHasher,
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
}

mod io {
    use super::*;
    use crate::patterns::delegate_factory_and_iter;
    use std::io;

    /// Produces each combination of a buffer state and a cursor position, except for those
    /// where the position is beyond the end of the buffer.
    impl<T: Exhaust + AsRef<[u8]> + Clone + fmt::Debug> Exhaust for io::Cursor<T> {
        type Iter = FlatZipMap<crate::Iter<T>, core::ops::RangeInclusive<u64>, io::Cursor<T>>;
        fn exhaust_factories() -> Self::Iter {
            FlatZipMap::new(
                T::exhaust(),
                |buf| 0..=(buf.as_ref().len() as u64),
                |buf, pos| {
                    let mut cursor = io::Cursor::new(buf);
                    cursor.set_position(pos);
                    cursor
                },
            )
        }
        factory_is_self!();
    }

    impl<T: io::Read + Exhaust> Exhaust for io::BufReader<T> {
        delegate_factory_and_iter!(T);
        fn from_factory(factory: Self::Factory) -> Self {
            io::BufReader::new(T::from_factory(factory))
        }
    }

    impl<T: io::Write + Exhaust> Exhaust for io::BufWriter<T> {
        delegate_factory_and_iter!(T);
        fn from_factory(factory: Self::Factory) -> Self {
            io::BufWriter::new(T::from_factory(factory))
        }
    }

    impl<T: io::Read + Exhaust, U: io::Read + Exhaust> Exhaust for io::Chain<T, U> {
        delegate_factory_and_iter!((T, U));

        fn from_factory(factory: Self::Factory) -> Self {
            let (first, second) = <(T, U)>::from_factory(factory);
            first.chain(second)
        }
    }

    impl Exhaust for io::Empty {
        type Iter = iter::Once<io::Empty>;
        fn exhaust_factories() -> Self::Iter {
            iter::once(io::empty())
        }
        factory_is_self!();
    }

    impl<T: io::Write + Exhaust> Exhaust for io::LineWriter<T> {
        delegate_factory_and_iter!(T);
        fn from_factory(factory: Self::Factory) -> Self {
            io::LineWriter::new(T::from_factory(factory))
        }
    }

    impl Exhaust for io::Repeat {
        delegate_factory_and_iter!(u8);
        fn from_factory(factory: Self::Factory) -> Self {
            io::repeat(factory)
        }
    }

    impl_singleton!([], io::Sink);
    impl_singleton!([], io::Stderr, io::stderr());
    impl_singleton!([], io::Stdin, io::stdin());
    impl_singleton!([], io::Stdout, io::stdout());

    // no impl for io::Take because it takes a 64-bit parameter
    // no impl for io::Error[Kind] because it is #[non_exhaustive]
    // no impl for io::SeekFrom because it takes a 64-bit parameter
}

mod sync {
    use super::*;
    use std::sync;

    impl_newtype_generic_indexable!(T: [], sync::Arc<T>, sync::Arc::new, Deref::deref);
    impl_newtype_generic_indexable!(T: [], Pin<sync::Arc<T>>, sync::Arc::pin, Deref::deref);

    impl_newtype_generic!(T: [], sync::Mutex<T>, sync::Mutex::new);
    impl_newtype_generic!(T: [], sync::RwLock<T>, sync::RwLock::new);

    impl<T: Exhaust> Exhaust for sync::OnceLock<T> {
        delegate_factory_and_iter!(Option<T>);

        fn from_factory(factory: Self::Factory) -> Self {
            let cell = sync::OnceLock::new();
            if let Some(value) = Option::<T>::from_factory(factory) {
                match cell.set(value) {
                    Ok(()) => {}
                    Err(_) => unreachable!(),
                }
            }
            cell
        }
    }

    impl_via_small_list!(
        sync::mpsc::RecvTimeoutError,
        [Self::Timeout, Self::Disconnected]
    );
    impl_via_small_list!(sync::mpsc::TryRecvError, [Self::Empty, Self::Disconnected]);
    impl_singleton!([], sync::mpsc::RecvError, sync::mpsc::RecvError);
    impl<T: Exhaust> Exhaust for sync::mpsc::TrySendError<T> {
        delegate_factory_and_iter!(remote::TrySendError<T>);
        fn from_factory(factory: Self::Factory) -> Self {
            match remote::TrySendError::from_factory(factory) {
                remote::TrySendError::Full(t) => Self::Full(t),
                remote::TrySendError::Disconnected(t) => Self::Disconnected(t),
            }
        }
    }
    impl_newtype_generic!(T: [], sync::mpsc::SendError<T>, sync::mpsc::SendError);

    // * sync::Condvar is stateful in a way we cannot handle.
    // * sync::Once could be implemented, but is very unlikely to be useful.
    // * sync::TryLockError could be implemented, but it doesn’t make sense to do so, since the
    //   thing it is expected to contain is a lock guard, which we cannot construct.

    mod remote {
        #![allow(missing_debug_implementations)]

        #[derive(crate::Exhaust)]
        pub enum TrySendError<T> {
            Full(T),
            Disconnected(T),
        }
    }
}
