use exhaust::Exhaust;

fn c<T: Exhaust>() -> Vec<T> {
    T::exhaust().collect()
}

#[derive(Clone, Debug, Exhaust, PartialEq)]
struct UnitStruct;

#[test]
fn unit_struct() {
    assert_eq!(c::<UnitStruct>(), vec![UnitStruct]);
}

#[derive(Clone, Debug, Exhaust, PartialEq)]
struct SimpleStruct {
    a: bool,
    b: bool,
}

#[test]
fn simple_struct() {
    assert_eq!(
        c::<SimpleStruct>(),
        vec![
            SimpleStruct { a: false, b: false },
            SimpleStruct { a: false, b: true },
            SimpleStruct { a: true, b: false },
            SimpleStruct { a: true, b: true },
        ]
    )
}
