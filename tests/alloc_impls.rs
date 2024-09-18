extern crate alloc;
use alloc::borrow::Cow;

use exhaust::Exhaust;

mod helper;
use helper::{check, check_double};

/// Test for Cow<SomeTypeThatImplementsClone>
#[test]
fn impl_cow_clone() {
    check_double::<Cow<'_, bool>>(vec![Cow::Owned(false), Cow::Owned(true)]);
}

/// Test for Cow with a type that does *not* implement Clone.
/// We can't do this with the usual ones like [`str`] because they're all unbounded DSTs.
/// (So, this implementation is unlikely to be very useful, really, but we might as well.)
#[test]
fn impl_cow_non_clone() {
    #[derive(Debug, PartialEq, Exhaust)]
    struct Foo(bool);

    #[derive(Debug, PartialEq, Exhaust)]
    struct FooOwned(Foo);

    impl alloc::borrow::Borrow<Foo> for FooOwned {
        fn borrow(&self) -> &Foo {
            &self.0
        }
    }
    impl alloc::borrow::ToOwned for Foo {
        type Owned = FooOwned;
        fn to_owned(&self) -> FooOwned {
            FooOwned(Foo(self.0))
        }
    }

    check::<Cow<'_, Foo>>(vec![
        Cow::Owned(FooOwned(Foo(false))),
        Cow::Owned(FooOwned(Foo(true))),
    ]);
}
