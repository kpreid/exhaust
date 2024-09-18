use core::hash;

use crate::patterns::impl_singleton;

impl_singleton!([H], hash::BuildHasherDefault<H>);
