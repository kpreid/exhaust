#![cfg_attr(not(target_has_atomic = "8"), allow(unused_imports, unused_macros))]

macro_rules! impl_atomic {
    ($t:ty, $atomic:ident) => {
        impl Exhaust for atomic::$atomic {
            $crate::patterns::delegate_factory_and_iter!($t);

            fn from_factory(factory: Self::Factory) -> Self {
                atomic::$atomic::new(factory)
            }
        }
    };
}

#[rustfmt::skip]
mod atomic_impl {
    use core::sync::atomic;
    use crate::Exhaust;

    #[cfg(target_has_atomic = "8")]  impl_atomic!(bool, AtomicBool);
    #[cfg(target_has_atomic = "8")]  impl_atomic!(i8, AtomicI8);
    #[cfg(target_has_atomic = "8")]  impl_atomic!(u8, AtomicU8);
    #[cfg(target_has_atomic = "16")] impl_atomic!(i16, AtomicI16);
    #[cfg(target_has_atomic = "16")] impl_atomic!(u16, AtomicU16);
    #[cfg(target_has_atomic = "32")] impl_atomic!(i32, AtomicI32);
    #[cfg(target_has_atomic = "32")] impl_atomic!(u32, AtomicU32);

    // * No `AtomicUsize` on the principle that it might be too large.
    // * No `AtomicPtr` on the principle that we don't produce nonsense pointers.
    // * No `Ordering` because it is `#[non_exhaustive]`.
}
