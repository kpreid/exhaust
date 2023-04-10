//! Unlike vectors, keyed collections (sets and maps) can be exhausted, since the key
//! space can be small. This module tests all such impls, since they are similar even
//! when the collections don't live in the same crate.

#![cfg(feature = "alloc")]

extern crate alloc;
use alloc::collections::{BTreeMap, BTreeSet};

use exhaust::Exhaust;

fn c<T: Exhaust>() -> Vec<T> {
    T::exhaust().collect()
}

#[test]
fn impl_btreeset() {
    assert_eq!(
        c::<BTreeSet<bool>>(),
        vec![
            BTreeSet::from_iter([]),
            BTreeSet::from_iter([false]),
            BTreeSet::from_iter([true]),
            BTreeSet::from_iter([false, true]),
        ]
    );
}

#[cfg(feature = "std")]
#[test]
fn impl_hashset() {
    use std::collections::HashSet;
    // TODO: This is not the preferred element ordering; [false, true] should be
    // before [true], as per lexicographic ordering. Fixing that will require
    // implementing our own powerset iterator.
    assert_eq!(
        c::<HashSet<bool>>(),
        vec![
            HashSet::from_iter([]),
            HashSet::from_iter([false]),
            HashSet::from_iter([true]),
            HashSet::from_iter([false, true]),
        ]
    );
}

fn bool_maps() -> Vec<BTreeMap<bool, bool>> {
    vec![
        BTreeMap::from_iter([]),
        BTreeMap::from_iter([(false, false)]),
        BTreeMap::from_iter([(false, true)]),
        BTreeMap::from_iter([(true, false)]),
        BTreeMap::from_iter([(true, true)]),
        BTreeMap::from_iter([(false, false), (true, false)]),
        BTreeMap::from_iter([(false, false), (true, true)]),
        BTreeMap::from_iter([(false, true), (true, false)]),
        BTreeMap::from_iter([(false, true), (true, true)]),
    ]
}

#[test]
fn impl_btreemap() {
    assert_eq!(c::<BTreeMap<bool, bool>>(), bool_maps(),);
}

#[cfg(feature = "std")]
#[test]
fn impl_hashmap() {
    use std::collections::HashMap;
    // Exhaustive iteration order currently depends on `HashSet` iteration order, so it
    // is not deterministic. Therefore, in order to check the results we have to ignore
    // order, and the easiest way to do that is to convert to BTree types.
    assert_eq!(
        HashMap::<bool, bool>::exhaust()
            .map(BTreeMap::from_iter)
            .collect::<BTreeSet<_>>(),
        bool_maps().into_iter().collect(),
    );
}
