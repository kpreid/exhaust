use crate::Exhaust;

/// Types whose values can be converted to and from consecutive integers.
///
/// If the total number of values of the type exceeds [`usize::MAX`], then compilation will fail
/// even if the trait implementation exists.
/// (Note that this is a more lenient rule than [`ExactSizeIterator`]’s; therefore, some generic
/// types may be able to implement [`Indexable`] but not
/// [`Exhaust<Iter: ExactSizeIterator>`][Exhaust].)
///
/// # Example
///
/// The indexing of an array of booleans works out to be identical to that of a binary number:
///
/// ```
/// use exhaust::Indexable;
///
/// assert_eq!(<[bool; 4]>::to_index(&[true, false, true, true]), 0b1011);
/// assert_eq!(<[bool; 4]>::from_index(0b0101), [false, true, false, true]);
/// ```
pub trait Indexable: Exhaust {
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
