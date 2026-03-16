use ::std::assert_eq;
use ::std::fmt;
use ::std::prelude::rust_2021::*;
use ::std::vec::Vec;

use ::exhaust::{Exhaust, Indexable};

/// All practical test cases are assumed to use fewer than this many explicit elements.
const LIMIT: usize = 1000;

#[allow(dead_code)] // compiled from multiple crates
#[track_caller]
pub(crate) fn check<T: Exhaust + fmt::Debug + PartialEq>(expected: Vec<T>) {
    assert!(expected.len() < LIMIT);

    let iter = T::exhaust();
    let size_hint = iter.size_hint();
    assert_eq!(iter.take(LIMIT).collect::<Vec<T>>(), expected);
    assert_size_hint_valid(size_hint, expected.len());
}

#[allow(dead_code)] // compiled from multiple crates
#[track_caller]
pub(crate) fn check_double<T: Exhaust + fmt::Debug + PartialEq>(mut expected: Vec<T>)
where
    T::Iter: DoubleEndedIterator,
{
    assert!(expected.len() < LIMIT);

    let fwd_iter = T::exhaust();
    let fwd_size_hint = fwd_iter.size_hint();
    assert_eq!(
        fwd_iter.take(LIMIT).collect::<Vec<T>>(),
        expected,
        "forward"
    );
    assert_size_hint_valid(fwd_size_hint, expected.len());

    expected.reverse();

    let rev_iter = T::exhaust().rev();
    let rev_size_hint = rev_iter.size_hint();
    assert_eq!(
        rev_iter.take(LIMIT).collect::<Vec<T>>(),
        expected,
        "reverse"
    );
    assert_size_hint_valid(rev_size_hint, expected.len());
}

/// Check that the [`Indexable`] implementation agrees with the [`Exhaust`] implementation.
///
/// This does not check any properties of the [`Exhaust`] implementation;
/// use it only together with [`check()`] or such.
#[allow(dead_code)] // compiled from multiple crates
#[track_caller]
pub(crate) fn check_indexable<T: Indexable + fmt::Debug + PartialEq>() {
    for (index_from_iter, value_from_iter) in T::exhaust().enumerate() {
        let index_from_trait = T::to_index(&value_from_iter);
        assert!(
            index_from_iter == index_from_trait,
            "T::to_index({value_from_iter:?}) returned {index_from_trait}, \
            but its position in iteration is {index_from_iter}"
        );
        let value_from_trait = T::from_index(index_from_iter);
        assert!(
            value_from_iter == value_from_trait,
            "T::from_index({index_from_iter}) returned {value_from_trait:?}, \
            but the value from iteration is {value_from_iter:?}"
        );
    }
}

#[track_caller]
pub(crate) fn assert_size_hint_valid((lower, upper): (usize, Option<usize>), expected_len: usize) {
    assert!(
        lower <= expected_len,
        "lower bound {lower} exceeds expected length {expected_len}",
    );
    assert!(
        upper.map_or(true, |upper| upper >= expected_len),
        "upper bound {upper:?} is less than expected length {expected_len}",
    );
}
