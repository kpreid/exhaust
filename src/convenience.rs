use crate::Exhaust;

/// Return an iterator over all values satisfying the given predicate.
///
/// This is equivalent to `T::exhaust().filter(filter)` but infers the value type.
///
/// TODO: Is this really part of the public API we want to offer?
pub fn brute_force_search<T, F>(filter: F) -> impl Iterator<Item = T>
where
    T: Exhaust,
    F: FnMut(&T) -> bool,
{
    T::exhaust().filter(filter)
}
