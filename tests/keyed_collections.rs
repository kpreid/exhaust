//! Unlike vectors, keyed collections (sets and maps) can be exhausted, since the key
//! space can be small. This module tests all such impls, since they are similar even
//! when the collections don't live in the same crate.

extern crate alloc;
use alloc::collections::{BTreeMap, BTreeSet};

use exhaust::Exhaust;

mod helper;
use helper::check;

/// Test [`BTreeSet`] (and the internal `ExhaustSet` algorithm) on [`bool`], a type with 2 values.
#[test]
fn impl_btreeset_2() {
    check::<BTreeSet<bool>>(vec![
        BTreeSet::from_iter([]),
        BTreeSet::from_iter([false]),
        BTreeSet::from_iter([false, true]),
        BTreeSet::from_iter([true]),
    ]);
}

/// Test [`BTreeSet`] (and the internal `ExhaustSet` algorithm) on an element type with 4 values,
/// and thus 2⁴ = 16 sets.
///
/// We don’t do this for `HashSet` because this test is to better exercise the internal algorithm,
/// which is shared between both set types.
#[test]
fn impl_btreeset_4() {
    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Exhaust)]
    enum Four {
        A,
        B,
        C,
        D,
    }
    use Four::*;

    check::<BTreeSet<Four>>(vec![
        BTreeSet::from_iter([]),
        BTreeSet::from_iter([A]),
        BTreeSet::from_iter([A, B]),
        BTreeSet::from_iter([A, B, C]),
        BTreeSet::from_iter([A, B, C, D]),
        BTreeSet::from_iter([A, B, D]),
        BTreeSet::from_iter([A, C]),
        BTreeSet::from_iter([A, C, D]),
        BTreeSet::from_iter([A, D]),
        BTreeSet::from_iter([B]),
        BTreeSet::from_iter([B, C]),
        BTreeSet::from_iter([B, C, D]),
        BTreeSet::from_iter([B, D]),
        BTreeSet::from_iter([C]),
        BTreeSet::from_iter([C, D]),
        BTreeSet::from_iter([D]),
    ]);
}

#[cfg(feature = "std")]
#[test]
fn impl_hashset() {
    use std::collections::HashSet;
    check::<HashSet<bool>>(vec![
        HashSet::from_iter([]),
        HashSet::from_iter([false]),
        HashSet::from_iter([false, true]),
        HashSet::from_iter([true]),
    ]);
}

/// The complex shape of an exhaustive list of sets only once, used twice.
fn bool_maps() -> Vec<BTreeMap<bool, bool>> {
    // TODO: This list (and the iterator it is testing) should be sorted, but is not.
    vec![
        BTreeMap::from_iter([]),
        BTreeMap::from_iter([(false, false)]),
        BTreeMap::from_iter([(false, true)]),
        BTreeMap::from_iter([(false, false), (true, false)]),
        BTreeMap::from_iter([(false, false), (true, true)]),
        BTreeMap::from_iter([(false, true), (true, false)]),
        BTreeMap::from_iter([(false, true), (true, true)]),
        BTreeMap::from_iter([(true, false)]),
        BTreeMap::from_iter([(true, true)]),
    ]
}

#[test]
fn impl_btreemap() {
    check::<BTreeMap<bool, bool>>(bool_maps());
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
            .take(1000) // precaution against infinite loop bugs
            .collect::<BTreeSet<_>>(),
        bool_maps().into_iter().collect(),
    );
}
