use core::{fmt, num, ops};

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
fn impl_f32() {
    // We can't exhaustively test it but we can check some easy properties.
    assert_eq!(f32::exhaust().next(), Some(0.0));
    assert!(f32::exhaust().next_back().unwrap().is_nan());
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
    check(vec![None, Some(false), Some(true)]);
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

mod impl_cell {
    use super::*;
    use core::cell::{Cell, OnceCell, RefCell, UnsafeCell};

    #[test]
    fn impl_cell() {
        check(vec![Cell::new(false), Cell::new(true)]);
    }

    #[test]
    fn impl_ref_cell() {
        check(vec![RefCell::new(false), RefCell::new(true)]);
    }

    #[test]
    fn impl_once_cell() {
        check(vec![
            OnceCell::new(),
            {
                let c = OnceCell::new();
                c.set(false).unwrap();
                c
            },
            {
                let c = OnceCell::new();
                c.set(true).unwrap();
                c
            },
        ]);

        // Since OnceCell is weird, let's separately check its actual values.
        assert_eq!(
            OnceCell::<bool>::exhaust()
                .map(|cell| cell.get().copied())
                .collect::<Vec<_>>(),
            vec![None, Some(false), Some(true)],
        );
    }

    #[test]
    fn impl_unsafe_cell() {
        // We can't use `check()` because `UnsafeCell` rightly does not implement `PartialEq`.

        assert_eq!(
            UnsafeCell::<bool>::exhaust()
                .map(UnsafeCell::into_inner)
                .collect::<Vec<_>>(),
            vec![false, true],
        );
    }
}

mod impl_fmt {
    use super::*;

    #[test]
    fn impl_alignment() {
        check(vec![
            core::fmt::Alignment::Left,
            core::fmt::Alignment::Right,
            core::fmt::Alignment::Center,
        ]);
    }

    #[test]
    fn impl_error() {
        check_double(vec![fmt::Error]);
    }
}

mod impl_ops {
    use super::*;

    #[test]
    fn impl_bound() {
        check(vec![
            ops::Bound::Included(false),
            ops::Bound::Included(true),
            ops::Bound::Excluded(false),
            ops::Bound::Excluded(true),
            ops::Bound::Unbounded,
        ]);
    }

    #[test]
    fn impl_control_flow() {
        check(vec![
            ops::ControlFlow::Continue(false),
            ops::ControlFlow::Continue(true),
            ops::ControlFlow::Break(false),
            ops::ControlFlow::Break(true),
        ]);
    }

    #[test]
    fn impl_range_from() {
        check(vec![false.., true..]);
    }

    #[test]
    fn impl_range_to() {
        check(vec![..false, ..true]);
    }

    #[test]
    fn impl_range_to_inclusive() {
        check(vec![..=false, ..=true]);
    }
}

/// Tests of `exhaust::Iter`, which isn't strictly an impl for crate core, but doesn't need its
/// own test target.
mod iter {
    #[test]
    fn size_hint_and_len() {
        let it = exhaust::Iter::<bool>::default();
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(it.len(), 2);
    }

    #[test]
    fn clone() {
        let mut it1 = exhaust::Iter::<bool>::default();
        assert_eq!(it1.next(), Some(false));
        let mut it2 = it1.clone();

        assert_eq!(it2.len(), 1);
        assert_eq!(it2.next(), Some(true));
    }
}
