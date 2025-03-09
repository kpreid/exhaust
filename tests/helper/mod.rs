use ::std::assert_eq;
use ::std::fmt;
use ::std::prelude::rust_2021::*;
use ::std::vec::Vec;

use ::exhaust::{Exhaust, Indexable};

// -------------------------------------------------------------------------------------------------

/// All practical test cases are assumed to use fewer than this many explicit elements.
const LIMIT: usize = 1000;

#[track_caller]
fn check_inner<T: Exhaust + fmt::Debug + PartialEq>(expected: &[T]) {
    assert!(expected.len() < LIMIT);

    let iter = T::exhaust();
    let size_hint = iter.size_hint();
    // TODO: also check the size hint on each step
    assert_eq!(
        iter.take(LIMIT).collect::<Vec<T>>(),
        expected,
        "forward iteration"
    );
    assert_size_hint_valid(size_hint, expected.len());
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

// -------------------------------------------------------------------------------------------------

/// Check correctness of an [`Exhaust`] implementation against explicitly listed values.
///
/// Does not check for [`DoubleEndedIterator`] or [`ExactSizeIterator`].
#[allow(dead_code)] // compiled from multiple crates
#[track_caller]
pub(crate) fn check<T: Exhaust + fmt::Debug + PartialEq>(expected: Vec<T>) {
    check_inner(&expected)
}

/// Check correctness of an [`Exhaust`] implementation against explicitly listed values.
///
/// Checks the [`ExactSizeIterator`] implementation.
/// Does not check [`DoubleEndedIterator`].
#[allow(dead_code)] // compiled from multiple crates
#[track_caller]
pub(crate) fn check_exact<T: Exhaust + fmt::Debug + PartialEq>(expected: Vec<T>)
where
    T::Iter: ExactSizeIterator,
{
    check_inner(&expected);
    assert_eq!(
        T::exhaust().len(),
        expected.len(),
        "len() does not match number of elements produced"
    );
}

/// Check correctness of an [`Exhaust`] implementation against explicitly listed values.
///
/// Checks the [`DoubleEndedIterator`] implementation.
/// Does not check [`ExactSizeIterator`].
#[allow(dead_code)] // compiled from multiple crates
#[track_caller]
pub(crate) fn check_double<T: Exhaust + fmt::Debug + PartialEq>(mut expected: Vec<T>)
where
    T::Iter: DoubleEndedIterator,
{
    check_inner::<T>(&expected);

    expected.reverse();
    assert_eq!(
        T::exhaust().rev().take(LIMIT).collect::<Vec<T>>(),
        expected,
        "reverse iteration"
    );
}

/// Check correctness of an [`Exhaust`] implementation against explicitly listed values.
///
/// Checks the [`DoubleEndedIterator`] and [`ExactSizeIterator`] implementations.
#[allow(dead_code)] // compiled from multiple crates
#[track_caller]
pub(crate) fn check_double_exact<T: Exhaust + fmt::Debug + PartialEq>(expected: Vec<T>)
where
    T::Iter: DoubleEndedIterator + ExactSizeIterator,
{
    let expected_len = expected.len();
    check_double(expected);
    assert_eq!(
        T::exhaust().len(),
        expected_len,
        "len() does not match number of elements produced"
    );
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
