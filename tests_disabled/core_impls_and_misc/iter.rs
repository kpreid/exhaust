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

#[test]
fn nth() {
    let mut it = exhaust::Iter::<u8>::default();
    assert_eq!(it.nth(100), Some(100));
    assert_eq!(it.next(), Some(101));
    assert_eq!(it.nth(1000), None);
}

#[test]
fn nth_back() {
    let mut it = exhaust::Iter::<u8>::default();
    assert_eq!(it.nth_back(10), Some(245));
    assert_eq!(it.next_back(), Some(244));
    assert_eq!(it.nth_back(1000), None);
}

#[test]
fn last_simple() {
    assert_eq!(exhaust::Iter::<u8>::default().last(), Some(255));
}

#[test]
fn last_mutated() {
    let mut it = exhaust::Iter::<u8>::default();
    it.next_back();
    assert_eq!(it.last(), Some(254));
}
