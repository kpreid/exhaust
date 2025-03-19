use crate::Exhaust;

/// Types whose values can be mapped to integer indices.
// TODO: Enable the ExactSizeIterator bound when we are done prototyping
pub trait Indexable: Exhaust /*<Iter: ExactSizeIterator>*/ {
    /// Number of distinct values of this type.
    /// Equivalent to `Self::exhaust().len()`, but is a constant.
    const VALUE_COUNT: usize;

    /// Returns the position within `Self::exhaust()` that `value` may be found.
    ///
    /// Equivalent to `Self::exhaust().position(value).unwrap()`, but more efficient;
    /// note that a correct implementation cannot panic.
    fn to_index(value: &Self) -> usize;

    /// Equivalent to `Self::exhaust().nth(index).unwrap()`.
    ///
    /// Panics if `index >= Self::VALUE_COUNT`.
    fn from_index(index: usize) -> Self;
}
