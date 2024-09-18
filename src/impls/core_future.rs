use core::future;

use crate::patterns::{impl_newtype_generic, impl_singleton};

impl_singleton!([T], future::Pending<T>, future::pending());
impl_newtype_generic!(T: [], future::Ready<T>, future::ready);
