extern crate alloc;
use alloc::borrow::Cow;

mod helper;
use helper::check_double;

#[test]
fn impl_cow() {
    check_double::<Cow<'_, bool>>(vec![Cow::Owned(false), Cow::Owned(true)]);
}
