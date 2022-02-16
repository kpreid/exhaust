use std::sync::Arc;

use crate::patterns::impl_newtype_generic;

impl_newtype_generic!(T: [], Arc<T>, Arc::new);
