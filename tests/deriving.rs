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
