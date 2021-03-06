use crate::patterns::{impl_newtype_generic, impl_via_array};

impl_newtype_generic!(T: [], core::cmp::Reverse<T>, core::cmp::Reverse);

impl_via_array!(
    core::cmp::Ordering,
    [Self::Less, Self::Equal, Self::Greater]
);
