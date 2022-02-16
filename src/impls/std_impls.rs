use core::pin::Pin;

use std::sync;

use crate::patterns::impl_newtype_generic;

impl_newtype_generic!(T: [], sync::Arc<T>, sync::Arc::new);
impl_newtype_generic!(T: [], Pin<sync::Arc<T>>, sync::Arc::pin);

// Mutex and RwLock are not Clone. This is evidence that we shouldn't have a Clone bound.
// impl_newtype_generic!(T: [], sync::Mutex<T>, sync::Mutex::new);
// impl_newtype_generic!(T: [], sync::RwLock<T>, sync::RwLock::new);

// Cannot implement Exhaust for sync::Once because it is not Clone.
