use crate::patterns::delegate_factory_and_iter;
use crate::Exhaust;

impl<T: Exhaust> Exhaust for Option<T> {
    delegate_factory_and_iter!(remote::Option<T>);

    fn from_factory(factory: Self::Factory) -> Self {
        match remote::Option::from_factory(factory) {
            remote::Option::None => None,
            remote::Option::Some(v) => Some(v),
        }
    }
}

/// Like the Serde “remote derive” pattern, we define a type imitating the real type
/// which the derive macro can process.
mod remote {
    #[allow(missing_debug_implementations)] // not actually public
    #[derive(crate::Exhaust)]
    pub enum Option<T> {
        None,
        Some(T),
    }
}
