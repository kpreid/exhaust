extern crate alloc;
use alloc::borrow::Cow;

use exhaust::Exhaust;

fn c<T: Exhaust>() -> Vec<T> {
    T::exhaust().collect()
}

#[test]
fn impl_cow() {
    assert_eq!(
        c::<Cow<'_, bool>>(),
        vec![Cow::Owned(false), Cow::Owned(true)]
    );
}
