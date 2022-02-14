use exhaust::Exhaust;
use std::fmt::Debug;

fn c<T: Debug + Exhaust>() -> Vec<T>
where
    <T as Exhaust>::Iter: Debug,
{
    let mut iterator = T::exhaust();
    let mut result = Vec::new();
    println!("Initial iterator state {:?}", iterator);
    while let Some(item) = iterator.next() {
        println!("{}. {:?} from {:?}", result.len(), item, iterator);
        if result.len() >= 10 {
            panic!(
                "exhaustive iterator didn't stop when expected;\nlast item: {:#?}\nstate: {:#?}",
                item, iterator
            );
        }
        result.push(item);
    }
    println!("Final iterator state {:?}", iterator);
    result
}

#[derive(Clone, Debug, Exhaust, PartialEq)]
struct UnitStruct;

#[test]
fn struct_unit() {
    assert_eq!(c::<UnitStruct>(), vec![UnitStruct]);
}

#[derive(Clone, Debug, Exhaust, PartialEq)]
struct SimpleStruct {
    // At least three fields are needed to check the carry logic.
    a: bool,
    b: bool,
    c: bool,
}

#[test]
#[rustfmt::skip]
fn struct_simple() {
    assert_eq!(
        c::<SimpleStruct>(),
        vec![
            SimpleStruct { a: false, b: false, c: false },
            SimpleStruct { a: false, b: false, c: true },
            SimpleStruct { a: false, b: true, c: false },
            SimpleStruct { a: false, b: true, c: true },
            SimpleStruct { a: true, b: false, c: false },
            SimpleStruct { a: true, b: false, c: true },
            SimpleStruct { a: true, b: true, c: false },
            SimpleStruct { a: true, b: true, c: true },
        ]
    )
}

#[derive(Clone, Debug, Exhaust, PartialEq)]
enum EmptyEnum {}

#[test]
fn enum_empty() {
    assert_eq!(c::<EmptyEnum>(), vec![]);
}

#[derive(Clone, Debug, Exhaust, PartialEq)]
enum OneValueEnum {
    Foo,
}

#[test]
fn enum_one_value() {
    assert_eq!(c::<OneValueEnum>(), vec![OneValueEnum::Foo]);
}

#[derive(Clone, Debug, Exhaust, PartialEq)]
enum FieldlessEnum {
    Foo,
    Bar,
    Baz,
}

#[test]
fn enum_fieldless_multi() {
    assert_eq!(
        c::<FieldlessEnum>(),
        vec![FieldlessEnum::Foo, FieldlessEnum::Bar, FieldlessEnum::Baz]
    );
}

#[derive(Clone, Debug, Exhaust, PartialEq)]
enum EnumWithFields {
    Foo(bool, bool),
    Bar(bool),
}

#[test]
fn enum_fields() {
    assert_eq!(
        c::<EnumWithFields>(),
        vec![
            EnumWithFields::Foo(false, false),
            EnumWithFields::Foo(false, true),
            EnumWithFields::Foo(true, false),
            EnumWithFields::Foo(true, true),
            EnumWithFields::Bar(false),
            EnumWithFields::Bar(true)
        ]
    );
}

#[allow(dead_code)]
#[derive(Clone, Exhaust)]
enum VariableNameHygieneTest {
    // These field names shouldn't conflict with internal variables in the generated impl.
    Foo { has_next: (), item: (), f0: () },
    Bar(()),
}
