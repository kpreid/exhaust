use crate::Exhaust;

pub fn brute_force_search<T, F>(filter: F) -> impl Iterator<Item = T>
where
    T: Exhaust,
    F: FnMut(&T) -> bool,
{
    T::exhaust().filter(filter)
}
