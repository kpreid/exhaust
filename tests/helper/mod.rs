use ::std::assert_eq;
use ::std::fmt;
use ::std::iter::FusedIterator;
use ::std::prelude::rust_2021::*;
use ::std::vec::Vec;

use ::exhaust::Exhaust;

// -------------------------------------------------------------------------------------------------

// TODO: Add a meta-test that demonstrates that this logic panics for all of the bugs
// it is trying to detect.

#[track_caller]
fn check_iter<T: fmt::Debug + PartialEq>(
    mut iterator: impl FusedIterator<Item = T> + fmt::Debug,
    expected: &[T],
) {
    let expected_len = expected.len();
    assert_size_hint_valid(iterator.size_hint(), expected.len());

    ::std::println!("Initial iterator state {iterator:?}");

    let mut i = 0;
    loop {
        assert_size_hint_valid(iterator.size_hint(), expected_len - i);

        let Some(item) = iterator.next() else {
            break;
        };
        ::std::println!("{i}. {item:?} from {iterator:?}");

        assert!(
            i < expected.len(),
            "iterator produced item {i}, {item:?}, beyond the {expected_len} expected items",
        );

        assert_eq!(item, expected[i], "item {i}");

        i += 1;
    }

    assert_eq!(
        i,
        expected_len,
        "iterator produced fewer items than expected; it is missing {missing:#?}",
        missing = &expected[i..],
    );

    // Check size hint when exhausted
    assert_size_hint_valid(iterator.size_hint(), 0);

    // Check `FusedIterator` behavior when exhausted
    assert_eq!(
        iterator.next(),
        None,
        "iterator should produce None after the end"
    );
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
    check_iter(T::exhaust(), &expected)
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
    // TODO: test correctness of len() throughout and not just at the beginning and end
    let mut iter = T::exhaust();
    let initial_len = iter.len();
    check_iter(&mut iter, &expected);

    // This assertion is done after checking the contents, because if both are wrong,
    // we’d rather see the wrong contents.
    assert_eq!(
        initial_len,
        expected.len(),
        "len() does not match number of elements produced"
    );
    assert_eq!(iter.len(), 0, "len() is not zero after iteration");
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
    check_iter::<T>(T::exhaust(), &expected);

    expected.reverse();
    check_iter::<T>(T::exhaust().rev(), &expected);
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
