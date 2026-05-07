//! Tests of implementations of [`Exhaust`] for [`core`] types.

use core::fmt;
use core::num;
use core::ops;

use exhaust::Exhaust;

use crate::helper::{check, check_double, check_double_exact};

#[test]
fn impl_unit() {
    check_double_exact(vec![()]);
    assert_eq!(size_of_val(&<()>::exhaust()), 1);
}

#[test]
fn impl_single_element_tuple() {
    check_double_exact(vec![(false,), (true,)]);
    assert_eq!(size_of_val(&<(bool,)>::exhaust()), 1);
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
    // Size is 3 inner iterators + 3 peeking states. (Would be nice to be smaller.)
    assert_eq!(size_of_val(&<(bool, bool, bool)>::exhaust()), 6);
}

#[test]
fn impl_phantom_data() {
    use core::marker::PhantomData;
    check_double_exact::<PhantomData<bool>>(vec![PhantomData]);
    assert_eq!(size_of_val(&<PhantomData<bool>>::exhaust()), 1);
}

/// [`core::convert::Infallible`] is not especially interesting in its role as an error type,
/// but it is also the only _uninhabited_ type in the standard library.
#[test]
fn impl_infallible() {
    check_double_exact(Vec::<core::convert::Infallible>::new());
    assert_eq!(size_of_val(&core::convert::Infallible::exhaust()), 0);
}

#[test]
fn impl_bool() {
    check_double_exact(vec![false, true]);
    assert_eq!(size_of_val(&<bool>::exhaust()), 1);
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
    // The non-u8 impls are macro-generated the same way as this one.
    check_double_exact(
        (1..=255)
            .map(|i| num::NonZeroU8::new(i).unwrap())
            .collect::<Vec<num::NonZeroU8>>(),
    );
}

#[test]
fn impl_nonzero_signed() {
    // The non-i8 impls are macro-generated the same way as this one.
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
fn impl_ordering() {
    use core::cmp::Ordering;
    check_double_exact(vec![Ordering::Less, Ordering::Equal, Ordering::Greater]);
    assert_eq!(size_of_val(&Ordering::exhaust()), 2);
}

#[test]
fn impl_option() {
    check(vec![None, Some(false), Some(true)]);
    assert_eq!(size_of_val(&<Option<bool>>::exhaust()), 1);
}

#[test]
fn impl_poll() {
    use core::task::Poll;
    check(vec![Poll::Pending, Poll::Ready(false), Poll::Ready(true)]);
    assert_eq!(size_of_val(&<Poll<bool>>::exhaust()), 1);
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
        check_double_exact(vec![Cell::new(false), Cell::new(true)]);
    }

    #[test]
    fn impl_ref_cell() {
        check_double_exact(vec![RefCell::new(false), RefCell::new(true)]);
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

mod impl_num {
    use super::*;

    #[test]
    fn impl_saturating() {
        use core::num::Saturating;
        // While using Saturating with Bool doesn’t make much sense, it is sufficient to
        // exercise exhaustion, since the struct has just the one public field.
        check_double_exact(vec![Saturating(false), Saturating(true)]);
    }

    #[test]
    fn impl_wrapping() {
        use core::num::Wrapping;
        // While using Wrapping with Bool doesn’t make much sense, it is sufficient to
        // exercise exhaustion, since the struct has just the one public field.
        check_double_exact(vec![Wrapping(false), Wrapping(true)]);
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
