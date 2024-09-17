use crate::Exhaust;

impl<T: Exhaust> Exhaust for Option<T> {
    type Iter = <remote::Option<T> as Exhaust>::Iter;
    type Factory = <remote::Option<T> as Exhaust>::Factory;

    fn exhaust_factories() -> Self::Iter {
        remote::Option::exhaust_factories()
    }

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
