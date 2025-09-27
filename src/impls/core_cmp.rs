use crate::patterns::{impl_newtype_generic_indexable, impl_via_small_list};

impl_newtype_generic_indexable!(T: [], core::cmp::Reverse<T>, core::cmp::Reverse, rev_get);
fn rev_get<T>(rev: &core::cmp::Reverse<T>) -> &T {
    &rev.0
}

impl_via_small_list!(
    core::cmp::Ordering,
    [Self::Less, Self::Equal, Self::Greater]
);
