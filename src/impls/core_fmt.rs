use core::fmt;

use crate::patterns::{delegate_factory_and_iter, impl_singleton};
use crate::Exhaust;

impl Exhaust for fmt::Alignment {
    delegate_factory_and_iter!(remote::Alignment);

    fn from_factory(factory: Self::Factory) -> Self {
        match remote::Alignment::from_factory(factory) {
            remote::Alignment::Left => fmt::Alignment::Left,
            remote::Alignment::Right => fmt::Alignment::Right,
            remote::Alignment::Center => fmt::Alignment::Center,
        }
    }
}

impl_singleton!([], fmt::Error);

/// Like the Serde “remote derive” pattern, we define a type imitating the real type
/// which the derive macro can process.
mod remote {
    #![allow(missing_debug_implementations)] // not actually public

    #[derive(crate::Exhaust)]
    pub enum Alignment {
        Left,
        Right,
        Center,
    }
}
