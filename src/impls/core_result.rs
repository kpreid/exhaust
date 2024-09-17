use crate::Exhaust;

impl<T: Exhaust, E: Exhaust> Exhaust for Result<T, E> {
    type Iter = <remote::Result<T, E> as Exhaust>::Iter;
    type Factory = <remote::Result<T, E> as Exhaust>::Factory;

    fn exhaust_factories() -> Self::Iter {
        remote::Result::exhaust_factories()
    }

    fn from_factory(factory: Self::Factory) -> Self {
        match remote::Result::from_factory(factory) {
            remote::Result::Ok(v) => Ok(v),
            remote::Result::Err(v) => Err(v),
        }
    }
}

/// Like the Serde “remote derive” pattern, we define a type imitating the real type
/// which the derive macro can process.
mod remote {
    #[allow(missing_debug_implementations)] // not actually public
    #[derive(crate::Exhaust)]
    pub enum Result<T, E> {
        Ok(T),
        Err(E),
    }
}
