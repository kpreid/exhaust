use exhaust::Exhaust;

#[derive(PartialEq, Debug, Exhaust)]
struct Foo {
    a: bool,
    b: Bar,
}

#[derive(PartialEq, Debug, Exhaust)]
enum Bar {
    One,
    Two(bool),
}

# #[rustfmt::skip]
assert_eq!(
    Foo::exhaust().collect::<Vec<Foo>>(),
    vec![
        Foo { a: false, b: Bar::One },
        Foo { a: false, b: Bar::Two(false) },
        Foo { a: false, b: Bar::Two(true) },
        Foo { a: true, b: Bar::One },
        Foo { a: true, b: Bar::Two(false) },
        Foo { a: true, b: Bar::Two(true) },
    ],
);
