use exhaust::Exhaust;

fn c<T: Exhaust>() -> Vec<T> {
    T::exhaust().collect()
}

#[test]
fn impl_unit() {
    assert_eq!(c::<()>(), vec![()]);
}

#[test]
fn impl_phantom_data() {
    use core::marker::PhantomData;
    assert_eq!(c::<PhantomData<bool>>(), vec![PhantomData]);
}

/// [`core::convert::Infallible`] is not especially interesting in its role as an error type,
/// but it is also the only _uninhabited_ type in the standard library.
#[test]
fn impl_infallible() {
    assert_eq!(
        c::<core::convert::Infallible>(),
        Vec::<core::convert::Infallible>::new()
    );
}

#[test]
fn impl_bool() {
    assert_eq!(c::<bool>(), vec![false, true]);
}

#[test]
fn impl_array_of_unit_type() {
    assert_eq!(c::<[(); 4]>(), vec![[(), (), (), ()]]);
}

#[test]
fn impl_array_of_uninhabited_type() {
    assert_eq!(
        c::<[core::convert::Infallible; 4]>(),
        Vec::<[core::convert::Infallible; 4]>::new()
    );
}

#[test]
fn impl_array_of_0() {
    assert_eq!(c::<[bool; 0]>(), vec![[]]);
}

#[test]
fn impl_array_of_1() {
    assert_eq!(c::<[bool; 1]>(), vec![[false], [true]]);
}

#[test]
fn impl_array_of_2() {
    assert_eq!(
        c::<[bool; 2]>(),
        vec![[false, false], [false, true], [true, false], [true, true]]
    );
}

#[test]
fn impl_array_of_3() {
    assert_eq!(
        c::<[bool; 3]>(),
        vec![
            [false, false, false],
            [false, false, true],
            [false, true, false],
            [false, true, true],
            [true, false, false],
            [true, false, true],
            [true, true, false],
            [true, true, true],
        ]
    );
}
