use core::num;

use exhaust::Exhaust;

mod helper;
use helper::{check, check_double};

#[test]
fn impl_unit() {
    check_double(vec![()]);
}

#[test]
fn impl_single_element_tuple() {
    check_double(vec![(false,), (true,)]);
}

#[test]
fn impl_nontrivial_tuple() {
    check(vec![
        (false, false, false),
        (false, false, true),
        (false, true, false),
        (false, true, true),
        (true, false, false),
        (true, false, true),
        (true, true, false),
        (true, true, true),
    ]);
}

#[test]
fn impl_phantom_data() {
    use core::marker::PhantomData;
    check_double::<PhantomData<bool>>(vec![PhantomData]);
}

/// [`core::convert::Infallible`] is not especially interesting in its role as an error type,
/// but it is also the only _uninhabited_ type in the standard library.
#[test]
fn impl_infallible() {
    check_double(Vec::<core::convert::Infallible>::new());
}

#[test]
fn impl_bool() {
    check_double(vec![false, true]);
}

#[test]
fn impl_char() {
    use std::collections::HashSet;
    let mut expected = HashSet::from([
        // Edge-case checking: endpoints of the valid range of char.
        '\u{0}',
        '\u{D7FF}',
        '\u{E000}',
        '\u{10FFFF}',
    ]);
    let mut count = 0;
    for c in char::exhaust() {
        expected.remove(&c);
        count += 1;
    }
    assert_eq!(expected, HashSet::new());
    assert_eq!(
        count,
        0x110000 // full numeric range...
        - 0x800 // ...but without surrogates
    );
}

#[test]
fn impl_nonzero_unsigned() {
    // The non-u8 impls are macro-generated the same way.
    check(
        (1..=255)
            .map(|i| num::NonZeroU8::new(i).unwrap())
            .collect::<Vec<num::NonZeroU8>>(),
    );
}

#[test]
fn impl_nonzero_signed() {
    // The non-i8 impls are macro-generated the same way.
    check(
        (-128..=127)
            .filter_map(num::NonZeroI8::new)
            .collect::<Vec<num::NonZeroI8>>(),
    );
}

#[test]
fn impl_array_of_unit_type() {
    check(vec![[(), (), (), ()]]);
}

#[test]
fn impl_array_of_uninhabited_type() {
    check(Vec::<[core::convert::Infallible; 4]>::new());
}

#[test]
fn impl_array_of_0() {
    check::<[bool; 0]>(vec![[]]);
}

#[test]
fn impl_array_of_1() {
    check::<[bool; 1]>(vec![[false], [true]]);
}

#[test]
fn impl_array_of_2() {
    check(vec![
        [false, false],
        [false, true],
        [true, false],
        [true, true],
    ]);
}

#[test]
fn impl_array_of_3() {
    check(vec![
        [false, false, false],
        [false, false, true],
        [false, true, false],
        [false, true, true],
        [true, false, false],
        [true, false, true],
        [true, true, false],
        [true, true, true],
    ]);
}

#[test]
fn impl_option() {
    check_double(vec![None, Some(false), Some(true)]);
}

#[test]
fn impl_poll() {
    use core::task::Poll;
    check(vec![Poll::Pending, Poll::Ready(false), Poll::Ready(true)]);
}

#[test]
fn impl_result() {
    check(vec![Ok(false), Ok(true), Err(false), Err(true)]);
}
