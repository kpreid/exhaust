use crate::patterns::delegate_factory_and_iter;
use crate::Exhaust;

impl<T: Exhaust, E: Exhaust> Exhaust for Result<T, E> {
    delegate_factory_and_iter!(remote::Result<T, E>);

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
