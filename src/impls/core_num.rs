use core::num;

use crate::patterns::{impl_newtype_generic, impl_via_array};

impl_via_array!(
    num::FpCategory,
    [
        Self::Nan,
        Self::Infinite,
        Self::Zero,
        Self::Subnormal,
        Self::Normal,
    ]
);

impl_newtype_generic!(T: [], num::Wrapping<T>, num::Wrapping);
