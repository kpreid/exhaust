use core::num;

use exhaust::Exhaust;

fn c<T: Exhaust>() -> Vec<T> {
    T::exhaust().collect()
}

#[test]
fn impl_unit() {
    assert_eq!(c::<()>(), vec![()]);
}

#[test]
fn impl_nontrivial_tuple() {
    assert_eq!(
        c::<(bool, bool, bool)>(),
        vec![
            (false, false, false),
            (false, false, true),
            (false, true, false),
            (false, true, true),
            (true, false, false),
            (true, false, true),
            (true, true, false),
            (true, true, true),
        ]
    );
}

#[test]
fn impl_phantom_data() {
    use core::marker::PhantomData;
    assert_eq!(c::<PhantomData<bool>>(), vec![PhantomData]);
}

/// [`core::convert::Infallible`] is not especially interesting in its role as an error type,
/// but it is also the only _uninhabited_ type in the standard library.
#[test]
fn impl_infallible() {
    assert_eq!(
        c::<core::convert::Infallible>(),
        Vec::<core::convert::Infallible>::new()
    );
}

#[test]
fn impl_bool() {
    assert_eq!(c::<bool>(), vec![false, true]);
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
    assert_eq!(
        c::<num::NonZeroU8>(),
        (1..=255)
            .map(|i| num::NonZeroU8::new(i).unwrap())
            .collect::<Vec<num::NonZeroU8>>()
    );
}

#[test]
fn impl_nonzero_signed() {
    // The non-u8 impls are macro-generated the same way.
    assert_eq!(
        c::<num::NonZeroI8>(),
        (-128..=127)
            .filter_map(num::NonZeroI8::new)
            .collect::<Vec<num::NonZeroI8>>()
    );
}

#[test]
fn impl_array_of_unit_type() {
    assert_eq!(c::<[(); 4]>(), vec![[(), (), (), ()]]);
}

#[test]
fn impl_array_of_uninhabited_type() {
    assert_eq!(
        c::<[core::convert::Infallible; 4]>(),
        Vec::<[core::convert::Infallible; 4]>::new()
    );
}

#[test]
fn impl_array_of_0() {
    assert_eq!(c::<[bool; 0]>(), vec![[]]);
}

#[test]
fn impl_array_of_1() {
    assert_eq!(c::<[bool; 1]>(), vec![[false], [true]]);
}

#[test]
fn impl_array_of_2() {
    assert_eq!(
        c::<[bool; 2]>(),
        vec![[false, false], [false, true], [true, false], [true, true]]
    );
}

#[test]
fn impl_array_of_3() {
    assert_eq!(
        c::<[bool; 3]>(),
        vec![
            [false, false, false],
            [false, false, true],
            [false, true, false],
            [false, true, true],
            [true, false, false],
            [true, false, true],
            [true, true, false],
            [true, true, true],
        ]
    );
}

#[test]
fn impl_option() {
    assert_eq!(c::<Option<bool>>(), vec![None, Some(false), Some(true)]);
}

#[test]
fn impl_result() {
    assert_eq!(
        c::<Result<bool, bool>>(),
        vec![Ok(false), Ok(true), Err(false), Err(true)]
    );
}
