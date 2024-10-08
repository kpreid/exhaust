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

/// Conflicts with type names the macro defines.
/// Ideally, there would be no such conflicts possible, but until then, this test will fail if
/// *change* what names are generated, reminding us to think about it.
///
/// TODO: Reduce conflicts by having the macro avoid using identifiers that appear in the input.
///
/// ```compile_fail
/// type ExhaustFooIter = ();
///
/// #[derive(exhaust::Exhaust)]
/// struct Foo(ExhaustFooIter);
/// ```
///
/// ```compile_fail
/// type ExhaustFooFactory = ();
///
/// #[derive(exhaust::Exhaust)]
/// struct Foo(ExhaustFooFactory);
/// ```
pub struct ConflictWithGeneratedTypeNames;
