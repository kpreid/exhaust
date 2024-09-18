// Import everything explicitly with nonstandard names (except for crates)
// to ensure that the macro is as hygienic as it can be.
#![no_implicit_prelude]

extern crate exhaust;
extern crate std;

mod helper;

use std::prelude::rust_2021 as p;

fn c<T: std::fmt::Debug + exhaust::Exhaust>() -> std::vec::Vec<T>
where
    <T as exhaust::Exhaust>::Iter: std::fmt::Debug,
{
    let mut iterator = T::exhaust();
    let mut result = std::vec::Vec::new();
    let size_hint = p::Iterator::size_hint(&iterator);
    std::println!("Initial iterator state {:?}", iterator);
    while let p::Some(item) = p::Iterator::next(&mut iterator) {
        std::println!("{}. {:?} from {:?}", result.len(), item, iterator);
        if result.len() >= 10 {
            std::panic!(
                "exhaustive iterator didn't stop when expected;\nlast item: {:#?}\nstate: {:#?}",
                item,
                iterator
            );
        }
        result.push(item);
    }
    std::println!("Final iterator state {:?}", iterator);

    helper::assert_size_hint_valid(size_hint, result.len());

    result
}

#[derive(Debug, exhaust::Exhaust, PartialEq)]
struct UnitStruct;

#[test]
fn struct_unit() {
    std::assert_eq!(c::<UnitStruct>(), std::vec![UnitStruct]);
}

#[derive(Debug, exhaust::Exhaust, PartialEq)]
struct SimpleStruct {
    // At least three fields are needed to check the carry logic.
    a: bool,
    b: bool,
    c: bool,
}

#[test]
fn struct_simple() {
    std::assert_eq!(
        c::<SimpleStruct>(),
        std::vec![
            SimpleStruct {
                a: false,
                b: false,
                c: false
            },
            SimpleStruct {
                a: false,
                b: false,
                c: true
            },
            SimpleStruct {
                a: false,
                b: true,
                c: false
            },
            SimpleStruct {
                a: false,
                b: true,
                c: true
            },
            SimpleStruct {
                a: true,
                b: false,
                c: false
            },
            SimpleStruct {
                a: true,
                b: false,
                c: true
            },
            SimpleStruct {
                a: true,
                b: true,
                c: false
            },
            SimpleStruct {
                a: true,
                b: true,
                c: true
            },
        ]
    )
}

#[derive(Debug, exhaust::Exhaust, PartialEq)]
struct GenericStruct<T> {
    a: T,
    b: T,
    // TODO: Also validate that trait bounds on the struct are handled
    // TODO: Test with lifetime and const generics.
}

#[test]
fn struct_generic() {
    std::assert_eq!(
        c::<GenericStruct<bool>>(),
        std::vec![
            GenericStruct { a: false, b: false },
            GenericStruct { a: false, b: true },
            GenericStruct { a: true, b: false },
            GenericStruct { a: true, b: true },
        ]
    )
}

#[derive(Debug, exhaust::Exhaust, PartialEq)]
enum EmptyEnum {}

#[test]
fn enum_empty() {
    std::assert_eq!(c::<EmptyEnum>(), std::vec![]);
}

#[derive(Debug, exhaust::Exhaust, PartialEq)]
enum OneValueEnum {
    Foo,
}

#[test]
fn enum_one_value() {
    std::assert_eq!(c::<OneValueEnum>(), std::vec![OneValueEnum::Foo]);
}

#[derive(Debug, exhaust::Exhaust, PartialEq)]
enum FieldlessEnum {
    Foo,
    Bar,
    Baz,
}

#[test]
fn enum_fieldless_multi() {
    std::assert_eq!(
        c::<FieldlessEnum>(),
        std::vec![FieldlessEnum::Foo, FieldlessEnum::Bar, FieldlessEnum::Baz]
    );
}

#[derive(Debug, exhaust::Exhaust, PartialEq)]
enum EnumWithFields {
    Foo(bool, bool),
    Bar(bool),
}

#[test]
fn enum_fields() {
    std::assert_eq!(
        c::<EnumWithFields>(),
        std::vec![
            EnumWithFields::Foo(false, false),
            EnumWithFields::Foo(false, true),
            EnumWithFields::Foo(true, false),
            EnumWithFields::Foo(true, true),
            EnumWithFields::Bar(false),
            EnumWithFields::Bar(true)
        ]
    );
}

#[derive(Debug, exhaust::Exhaust, PartialEq)]
enum EnumWithGeneric<T> {
    N,
    S(T),
}

#[test]
fn enum_generic() {
    std::assert_eq!(
        c::<EnumWithGeneric<bool>>(),
        std::vec![
            EnumWithGeneric::N,
            EnumWithGeneric::S(false),
            EnumWithGeneric::S(true),
        ]
    );
}

mod module {
    #[derive(::exhaust::Exhaust)]
    enum EnumInsideMod<T> {
        N,
        S(T),
    }

    #[derive(Debug, PartialEq, ::exhaust::Exhaust)]
    struct StructInsideMod(bool);
}

/// Items in functions have different scoping rules than items in modules.
/// Exercise using the derive inside one.
#[test]
fn function_containing_derive() {
    #[derive(exhaust::Exhaust)]
    enum EnumInsideFn<T> {
        N,
        S(T),
    }

    #[derive(Debug, PartialEq, exhaust::Exhaust)]
    struct StructInsideFn(bool);

    std::assert_eq!(
        c::<StructInsideFn>(),
        std::vec![StructInsideFn(false), StructInsideFn(true)]
    );
}

#[allow(dead_code)]
#[derive(exhaust::Exhaust)]
enum VariableNameHygieneTest {
    // These field and variant names shouldn't conflict with internal variables in the generated impl.
    Foo { has_next: (), item: (), f0: () },
    Bar(()),
    Done,
}

#[test]
fn debug_impls() {
    use exhaust::Exhaust;
    use std::{assert_eq, format, iter::Iterator};

    #[derive(Debug, PartialEq, Exhaust)]
    enum Foo {
        X,
        Y,
    }

    // TODO: It would be better to print some state, but that requires more code generation work.
    let mut iter = Foo::exhaust_factories();
    assert_eq!(format!("{iter:?}"), "ExhaustFooIter { .. }");
    let factory = Iterator::next(&mut iter).unwrap();
    assert_eq!(format!("{factory:?}"), "ExhaustFooFactory { .. }");
    assert_eq!(Foo::from_factory(factory), Foo::X);
}

/// The presence of this trait's methods should not disrupt the generated code
#[allow(dead_code)]
trait ConfusingTraitInScope {
    fn next(&self, _dont_call_me: ()) {}
    fn peekable(&self, _dont_call_me: ()) {}
    fn clone(&self, _dont_call_me: ()) {}
    fn unwrap(&self, _dont_call_me: ()) {}
    fn default(_dont_call_me: ()) {}
}
impl<T: ?p::Sized> ConfusingTraitInScope for T {}

#[allow(dead_code)]
trait Exhaust {
    // Not the real Exhaust trait
}
