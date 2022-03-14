//! Unlike vectors, keyed collections (sets and maps) can be exhausted, since the key
//! space can be small. This module tests all such impls, since they are similar even
//! when the collections don't live in the same crate.

#![cfg(feature = "alloc")]

use exhaust::Exhaust;

fn c<T: Exhaust>() -> Vec<T> {
    T::exhaust().collect()
}

#[test]
fn impl_btreeset() {
    use std::collections::BTreeSet;
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

#[test]
fn impl_btreemap() {
    use std::collections::BTreeMap;
    assert_eq!(
        c::<BTreeMap<bool, bool>>(),
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
    );
}
