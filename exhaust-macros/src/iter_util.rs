//! Iteration helpers; basically a tiny replacement for `itertools` to minimize build-time
//! dependencies.

/// Merge 5 iterators into an iterator of 5-element tuples.
pub(crate) fn zip5<I1, I2, I3, I4, I5>(
    it1: I1,
    it2: I2,
    it3: I3,
    it4: I4,
    it5: I5,
) -> impl Iterator<Item = (I1::Item, I2::Item, I3::Item, I4::Item, I5::Item)>
where
    I1: IntoIterator,
    I2: IntoIterator,
    I3: IntoIterator,
    I4: IntoIterator,
    I5: IntoIterator,
{
    it1.into_iter()
        .zip(it2)
        .zip(it3)
        .zip(it4)
        .zip(it5)
        .map(|((((i1, i2), i3), i4), i5)| (i1, i2, i3, i4, i5))
}

/// Collect 6-element tuples into 6 containers.
#[must_use]
pub(crate) fn unzip6<T1, T2, T3, T4, T5, T6, C1, C2, C3, C4, C5, C6>(
    iterator: impl Iterator<Item = (T1, T2, T3, T4, T5, T6)>,
) -> (C1, C2, C3, C4, C5, C6)
where
    C1: Default + Extend<T1>,
    C2: Default + Extend<T2>,
    C3: Default + Extend<T3>,
    C4: Default + Extend<T4>,
    C5: Default + Extend<T5>,
    C6: Default + Extend<T6>,
{
    let mut c1 = C1::default();
    let mut c2 = C2::default();
    let mut c3 = C3::default();
    let mut c4 = C4::default();
    let mut c5 = C5::default();
    let mut c6 = C6::default();

    iterator.for_each(|(i1, i2, i3, i4, i5, i6)| {
        c1.extend([i1]);
        c2.extend([i2]);
        c3.extend([i3]);
        c4.extend([i4]);
        c5.extend([i5]);
        c6.extend([i6]);
    });

    (c1, c2, c3, c4, c5, c6)
}

/// Collect 7-element tuples into 7 containers.
#[must_use]
pub(crate) fn unzip7<T1, T2, T3, T4, T5, T6, T7, C1, C2, C3, C4, C5, C6, C7>(
    iterator: impl Iterator<Item = (T1, T2, T3, T4, T5, T6, T7)>,
) -> (C1, C2, C3, C4, C5, C6, C7)
where
    C1: Default + Extend<T1>,
    C2: Default + Extend<T2>,
    C3: Default + Extend<T3>,
    C4: Default + Extend<T4>,
    C5: Default + Extend<T5>,
    C6: Default + Extend<T6>,
    C7: Default + Extend<T7>,
{
    let mut c1 = C1::default();
    let mut c2 = C2::default();
    let mut c3 = C3::default();
    let mut c4 = C4::default();
    let mut c5 = C5::default();
    let mut c6 = C6::default();
    let mut c7 = C7::default();

    iterator.for_each(|(i1, i2, i3, i4, i5, i6, i7)| {
        c1.extend([i1]);
        c2.extend([i2]);
        c3.extend([i3]);
        c4.extend([i4]);
        c5.extend([i5]);
        c6.extend([i6]);
        c7.extend([i7]);
    });

    (c1, c2, c3, c4, c5, c6, c7)
}
