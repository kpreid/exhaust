extern crate std;
use std::prelude::rust_2021::*;
use std::vec;

use crate::Exhaust;

fn c<T: Exhaust>() -> Vec<T> {
    T::exhaust().collect()
}

#[test]
fn impl_unit() {
    assert_eq!(c::<()>(), vec![()]);
}

#[test]
fn impl_bool() {
    assert_eq!(c::<bool>(), vec![false, true]);
}

#[test]
fn impl_array_empty() {
    assert_eq!(c::<[bool; 0]>(), Vec::<[bool; 0]>::new());
}

#[test]
fn impl_array_singular_valuie() {
    assert_eq!(c::<[(); 4]>(), vec![[(), (), (), ()]]);
}

#[test]
fn impl_array_nonempty() {
    assert_eq!(
        c::<[bool; 2]>(),
        vec![[false, false], [false, true], [true, false], [true, true]]
    );
}
