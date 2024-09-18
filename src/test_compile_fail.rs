//! Doc-tests not visible as part of the documentation.

/// It should not be possible to access the fields/variants of a derived factory type.
/// (This test isn't very good since it's trying to prove a negative with a guess...)
///
/// ```no_run
/// type _Foo = <Option<()> as ::exhaust::Exhaust>::Factory;
/// ```
///
/// ```compile_fail
/// type _Foo = <Option<()> as ::exhaust::Exhaust>::Factory;
/// _Foo::None;
/// ```
pub struct OptionFactoryNotAccessible;
