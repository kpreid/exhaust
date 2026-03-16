use core::cmp::Ordering;

use crate::patterns::{impl_newtype_generic_indexable, impl_via_array};

impl_newtype_generic_indexable!(T: [], core::cmp::Reverse<T>, core::cmp::Reverse, rev_get);
fn rev_get<T>(rev: &core::cmp::Reverse<T>) -> &T {
    &rev.0
}

impl_via_array!(
    Ordering,
    [Ordering::Less, Ordering::Equal, Ordering::Greater]
);
