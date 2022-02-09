Exhaust(ive iteration for Rust)
===============================

`exhaust` is a Rust library which provides the `Exhaust` trait, which can be used to iterate over **all possible values of a type** that implements it. Implementations are provided for standard library types, and derive macros are available to allow easy implementation for user-defined types.

Exhaustive iteration may be useful for exhaustive testing, working with enums, and solving constraints by brute-force search.

`exhaust` is `no_std` compatible with default features disabled. The `alloc` and `std` features add implementations for the corresponding standard library crates.

Project status and stability
----------------------------

`exhaust` is currently in an early stage of development, but I intend to quickly bring it to feature-completeness within its narrow scope, making it a library that can be relied upon.

License
-------

TODO: Write MIT/Apache license statement
