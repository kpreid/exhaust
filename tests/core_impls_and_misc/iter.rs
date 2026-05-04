//! Tests of [`exhaust::Iter`].

#[test]
fn size_hint_and_len() {
    let it = exhaust::Iter::<bool>::default();
    assert_eq!(it.size_hint(), (2, Some(2)));
    assert_eq!(it.len(), 2);
}

#[test]
fn clone() {
    let mut it1 = exhaust::Iter::<bool>::default();
    assert_eq!(it1.next(), Some(false));
    let mut it2 = it1.clone();

    assert_eq!(it2.len(), 1);
    assert_eq!(it2.next(), Some(true));
}
