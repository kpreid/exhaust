use std::iter;

use itertools::izip;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens as _};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, DeriveInput};

mod common;
use common::ExhaustContext;

mod fields;
use fields::{exhaust_iter_fields, ExhaustFields};

use crate::common::ConstructorSyntax;

// Note: documentation is on the reexport so that it can have working links.
#[proc_macro_derive(Exhaust)]
pub fn derive_exhaust(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_impl(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Generate an impl of Exhaust for a built-in tuple type.
/// This macro is only useful within the `exhaust` crate.
#[proc_macro]
#[doc(hidden)]
pub fn impl_exhaust_for_tuples(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitInt);
    tuple_impls_up_to(input.base10_parse().unwrap())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn derive_impl(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let DeriveInput {
        ident: item_type_name,
        attrs: _,
        vis,
        generics,
        data,
    } = input;

    let item_type_name_str = &item_type_name.to_string();
    let factory_type_name = common::generated_type_name(item_type_name_str, "Factory");
    let iterator_type_name = common::generated_type_name(item_type_name_str, "Iter");

    let ctx = ExhaustContext {
        vis,
        generics,
        iterator_type_name,
        item_type: ConstructorSyntax::Braced(item_type_name.to_token_stream()),
        factory_type: ConstructorSyntax::Braced(factory_type_name.to_token_stream()),
        exhaust_crate_path: syn::parse_quote! { ::exhaust },
    };
    let ExhaustContext {
        iterator_type_name,
        exhaust_crate_path,
        ..
    } = &ctx;

    let (iterator_and_factory_decl, from_factory_body) = match data {
        syn::Data::Struct(s) => exhaust_iter_struct(s, &ctx),
        syn::Data::Enum(e) => exhaust_iter_enum(e, &ctx),
        syn::Data::Union(syn::DataUnion { union_token, .. }) => Err(syn::Error::new(
            union_token.span,
            "derive(Exhaust) does not support unions",
        )),
    }?;

    let (impl_generics, ty_generics, augmented_where_predicates) =
        ctx.generics_with_bounds(syn::parse_quote! {});

    Ok(quote! {
        // rust-analyzer (but not rustc) sometimes produces lints on macro generated code it
        // shouldn't. We don't expect to actually hit this case normally, but in general,
        // we don't want to *ever* bother our users with unfixable warnings about weird names.
        #[allow(nonstandard_style)]
        // This anonymous constant allows us to make all our generated types be public-in-private,
        // without altering the meaning of any paths they use.
        const _: () = {
            impl #impl_generics #exhaust_crate_path::Exhaust for #item_type_name #ty_generics
            where #augmented_where_predicates {
                type Iter = #iterator_type_name #ty_generics;
                type Factory = #factory_type_name #ty_generics;
                fn exhaust_factories() -> Self::Iter {
                    ::core::default::Default::default()
                }
                fn from_factory(factory: Self::Factory) -> Self {
                    #from_factory_body
                }
            }

            #iterator_and_factory_decl
        };
    })
}

fn tuple_impls_up_to(size: u64) -> Result<TokenStream2, syn::Error> {
    (2..=size).map(tuple_impl).collect()
}

/// Generate an impl of Exhaust for a built-in tuple type.
///
/// This is almost but not quite identical to [`exhaust_iter_struct`], due to the syntax
/// of tuples and due to it being used from the same crate (so that access is via
/// crate::Exhaust instead of ::exhaust::Exhaust).
fn tuple_impl(size: u64) -> Result<TokenStream2, syn::Error> {
    if size < 2 {
        return Err(syn::Error::new(
            Span::call_site(),
            "tuple type of size less than 2 not supported",
        ));
    }

    let value_type_vars: Vec<Ident> = (0..size)
        .map(|i| Ident::new(&format!("T{i}"), Span::mixed_site()))
        .collect();
    let factory_value_vars: Vec<Ident> = (0..size)
        .map(|i| Ident::new(&format!("factory{i}"), Span::mixed_site()))
        .collect();
    let synthetic_fields: syn::Fields = syn::Fields::Unnamed(syn::FieldsUnnamed {
        paren_token: syn::token::Paren(Span::mixed_site()),
        unnamed: value_type_vars
            .iter()
            .map(|type_var| syn::Field {
                attrs: vec![],
                vis: parse_quote! { pub },
                mutability: syn::FieldMutability::None,
                ident: None,
                colon_token: None,
                ty: syn::Type::Verbatim(type_var.to_token_stream()),
            })
            .collect(),
    });

    // Synthesize a good-enough context to use the derive tools.
    let ctx: ExhaustContext = ExhaustContext {
        vis: parse_quote! { pub },
        generics: syn::Generics {
            lt_token: None,
            params: value_type_vars
                .iter()
                .map(|var| {
                    syn::GenericParam::Type(syn::TypeParam {
                        attrs: vec![],
                        ident: var.clone(),
                        colon_token: None,
                        bounds: Punctuated::default(),
                        eq_token: None,
                        default: None,
                    })
                })
                .collect(),
            gt_token: None,
            where_clause: None,
        },
        item_type: ConstructorSyntax::Tuple,
        factory_type: ConstructorSyntax::Tuple,
        iterator_type_name: common::generated_type_name("Tuple", "Iter"),
        exhaust_crate_path: parse_quote! { crate },
    };

    let iterator_type_name = &ctx.iterator_type_name;

    // Generate the field-exhausting iteration logic
    let ExhaustFields {
        state_field_decls,
        factory_field_decls: _, // unused because we use tuples instead
        initializers,
        cloners,
        field_pats,
        advance,
    } = exhaust_iter_fields(
        &ctx,
        &synthetic_fields,
        &quote! {},
        &ConstructorSyntax::Tuple,
    );

    let iterator_impls = ctx.impl_iterator_and_factory_traits(
        quote! {
            match self {
                Self { #field_pats } => {
                    #advance
                }
            }
        },
        quote! { Self { #initializers } },
        quote! {
            let Self { #field_pats } = self;
            Self { #cloners }
        },
    );

    let iterator_doc = ctx.iterator_doc();

    Ok(quote! {
        const _: () = {
            impl<#( #value_type_vars , )*> crate::Exhaust for ( #( #value_type_vars , )* )
            where #( #value_type_vars : crate::Exhaust, )*
            {
                type Iter = #iterator_type_name <#( #value_type_vars , )*>;
                type Factory = (#(
                    <#value_type_vars as crate::Exhaust>::Factory,
                )*);
                fn exhaust_factories() -> Self::Iter {
                    ::core::default::Default::default()
                }
                fn from_factory(factory: Self::Factory) -> Self {
                    let (#( #factory_value_vars , )*) = factory;
                    (#(
                        <#value_type_vars as crate::Exhaust>::from_factory(#factory_value_vars),
                    )*)
                }
            }

            #[doc = #iterator_doc]
            pub struct #iterator_type_name <#( #value_type_vars , )*>
            where #( #value_type_vars : crate::Exhaust, )*
            {
                #state_field_decls
            }

            #iterator_impls
        };
    })
}

fn exhaust_iter_struct(
    s: syn::DataStruct,
    ctx: &ExhaustContext,
) -> Result<(TokenStream2, TokenStream2), syn::Error> {
    let vis = &ctx.vis;
    let exhaust_crate_path = &ctx.exhaust_crate_path;
    let (impl_generics, ty_generics, augmented_where_predicates) =
        ctx.generics_with_bounds(syn::parse_quote! {});
    let iterator_type_name = &ctx.iterator_type_name;
    let factory_type_name = &ctx.factory_type.path()?;
    let factory_type = &ctx.factory_type.parameterized(&ctx.generics);

    let factory_state_struct_type = ctx.generated_type_name("FactoryState")?;
    let factory_state_ctor = ConstructorSyntax::Braced(factory_state_struct_type.to_token_stream());

    let ExhaustFields {
        state_field_decls,
        factory_field_decls,
        initializers,
        cloners,
        field_pats,
        advance,
    } = if s.fields.is_empty() {
        let empty_state_expr = factory_state_ctor.value_expr([].iter(), [].iter());
        ExhaustFields {
            state_field_decls: quote! { done: bool, },
            factory_field_decls: syn::Fields::Unit,
            initializers: quote! { done: false, },
            cloners: quote! { done: *done, },
            field_pats: quote! { done, },
            advance: quote! {
                if *done {
                    ::core::option::Option::None
                } else {
                    *done = true;
                    ::core::option::Option::Some(#factory_type_name(#empty_state_expr))
                }
            },
        }
    } else {
        exhaust_iter_fields(
            ctx,
            &s.fields,
            ctx.factory_type.path()?,
            &factory_state_ctor,
        )
    };

    // Note: The iterator must have trait bounds because its fields, being of type
    // `<SomeOtherTy as Exhaust>::Iter`, require that `SomeOtherTy: Exhaust`.

    let impls = ctx.impl_iterator_and_factory_traits(
        quote! {
            match self {
                Self { #field_pats } => {
                    #advance
                }
            }
        },
        quote! { Self { #initializers } },
        quote! {
            let Self { #field_pats } = self;
            Self { #cloners }
        },
    );

    let factory_struct_clone_arm = common::clone_like_struct_conversion(
        &s.fields,
        factory_state_ctor.path()?,
        factory_state_ctor.path()?,
        &quote! { ref },
        |expr| quote! { ::core::clone::Clone::clone(#expr) },
    );

    let factory_to_self_transform = common::clone_like_struct_conversion(
        &s.fields,
        factory_state_ctor.path()?,
        ctx.item_type.path()?,
        &quote! {},
        |expr| quote! { #exhaust_crate_path::Exhaust::from_factory(#expr) },
    );

    // Generate factory state struct with the same syntax type as the original
    // (for elegance, not because it matters functionally).
    // This struct is always wrapped in a newtype struct to hide implementation details reliably.
    let factory_state_struct_decl = match &factory_field_decls {
        syn::Fields::Unit | syn::Fields::Unnamed(_) => quote! {
            #vis struct #factory_state_struct_type #ty_generics #factory_field_decls
            where #augmented_where_predicates;

        },

        syn::Fields::Named(_) => quote! {
            #vis struct #factory_state_struct_type #ty_generics
            where #augmented_where_predicates
            #factory_field_decls
        },
    };

    Ok((
        quote! {
            // Struct that is exposed as the `<Self as Exhaust>::Iter` type.
            // A wrapper struct is not needed because it always has at least one private field.
            #vis struct #iterator_type_name #ty_generics
            where #augmented_where_predicates {
                #state_field_decls
            }

            // Struct that is exposed as the `<Self as Exhaust>::Factory` type.
            #vis struct #factory_type_name #ty_generics (#factory_state_struct_type #ty_generics)
            where #augmented_where_predicates;

            #impls

            #factory_state_struct_decl

            // A manual impl of Clone is required to *not* have a `Clone` bound on the generics.
            impl #impl_generics ::core::clone::Clone for #factory_type
            where #augmented_where_predicates {
                fn clone(&self) -> Self {
                    Self(match self.0 {
                        #factory_struct_clone_arm
                    })
                }
            }

        },
        quote! {
            match factory.0 {
                #factory_to_self_transform
            }
        },
    ))
}

fn exhaust_iter_enum(
    e: syn::DataEnum,
    ctx: &ExhaustContext,
) -> Result<(TokenStream2, TokenStream2), syn::Error> {
    let vis = &ctx.vis;
    let exhaust_crate_path = &ctx.exhaust_crate_path;
    let iterator_type_name = &ctx.iterator_type_name;
    let factory_outer_type_path = &ctx.factory_type.path()?;
    let factory_type = &ctx.factory_type.parameterized(&ctx.generics);

    // These enum types are both wrapped in structs,
    // so that the user of the macro cannot depend on its implementation details.
    let iter_state_enum_type = ctx.generated_type_name("IterState")?;
    let factory_state_enum_type = ctx.generated_type_name("FactoryState")?.to_token_stream();
    let factory_state_ctor = ConstructorSyntax::Braced(factory_state_enum_type.clone());

    // One ident per variant of the original enum.
    let state_enum_progress_variants: Vec<Ident> = e
        .variants
        .iter()
        .map(|v| {
            // Renaming the variant serves two purposes: less confusing error/debug text,
            // and disambiguating from the “Done” variant.
            Ident::new(&format!("Exhaust{}", v.ident), v.span())
        })
        .collect();

    // TODO: ensure no name conflict, perhaps by renaming the others
    let done_variant = Ident::new("Done", Span::mixed_site());

    // All variants of our generated enum, which are equal to the original enum
    // plus a "done" variant.
    #[allow(clippy::type_complexity)]
    let (
        state_enum_variant_decls,
        state_enum_variant_initializers,
        state_enum_variant_cloners,
        state_enum_field_pats,
        state_enum_variant_advancers,
        mut factory_variant_decls,
    ): (
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
    ) = itertools::multiunzip(e
        .variants
        .iter()
        .zip(state_enum_progress_variants.iter())
        .map(|(target_variant, state_ident)| {
            let target_variant_ident = &target_variant.ident;
            let fields::ExhaustFields {
                state_field_decls,
                factory_field_decls,
                initializers: state_fields_init,
                cloners: state_fields_clone,
                field_pats,
                advance,
            } = if target_variant.fields.is_empty() {
                // TODO: don't even construct this dummy value (needs refactoring)
                fields::ExhaustFields {
                    state_field_decls: quote! {},
                    factory_field_decls: syn::Fields::Unit,
                    initializers: quote! {},
                    cloners: quote! {},
                    field_pats: quote! {},
                    advance: quote! {
                        compile_error!("can't happen: fieldless ExhaustFields not used")
                    },
                }
            } else {
                fields::exhaust_iter_fields(
                    ctx,
                    &target_variant.fields,
                    factory_outer_type_path,
                    &factory_state_ctor.with_variant(target_variant_ident),
                )
            };

            (
                quote! {
                    #state_ident {
                        #state_field_decls
                    }
                },
                quote! {
                    #iter_state_enum_type :: #state_ident { #state_fields_init }
                },
                quote! {
                    #iter_state_enum_type :: #state_ident { #field_pats } =>
                        #iter_state_enum_type :: #state_ident { #state_fields_clone }
                },
                field_pats,
                advance,
                quote! {
                    #target_variant_ident #factory_field_decls
                },
            )
        })
        .chain(iter::once((
            done_variant.to_token_stream(),
            quote! {
                // iterator construction
                #iter_state_enum_type :: #done_variant {}
            },
            quote! {
                // clone() match arm
                #iter_state_enum_type :: #done_variant {} => #iter_state_enum_type :: #done_variant {}
            },
            quote! {},
            quote! { compile_error!("done advancer not used") },
            quote! { compile_error!("done factory variant not used") },
        ))));

    factory_variant_decls.pop(); // no Done arm in the factory enum

    let first_state_variant_initializer = &state_enum_variant_initializers[0];

    // Match arms to advance the iterator.
    let variant_next_arms = izip!(
        e.variants.iter(),
        state_enum_progress_variants.iter(),
        state_enum_field_pats.iter(),
        state_enum_variant_initializers.iter().skip(1),
        state_enum_variant_advancers.iter(),
    )
    .map(
        |(target_enum_variant, state_ident, pats, next_state_initializer, field_advancer)| {
            let target_variant_ident = &target_enum_variant.ident;
            let advancer = if target_enum_variant.fields.is_empty() {
                let factory_state_expr = factory_state_ctor
                    .with_variant(target_variant_ident)
                    .value_expr([].iter(), [].iter());
                quote! {
                    self.0 = #next_state_initializer;
                    ::core::option::Option::Some(#factory_outer_type_path(#factory_state_expr))
                }
            } else {
                quote! {
                    let maybe_variant = { #field_advancer };
                    match maybe_variant {
                        ::core::option::Option::Some(v) => ::core::option::Option::Some(v),
                        ::core::option::Option::None => {
                            self.0 = #next_state_initializer;
                            // TODO: recursion is a kludge here; rewrite as loop{}
                            ::core::iter::Iterator::next(self)
                        }
                    }
                }
            };
            quote! {
                #iter_state_enum_type::#state_ident { #pats } => {
                    #advancer
                }
            }
        },
    );

    let factory_enum_variant_clone_arms: Vec<TokenStream2> = common::clone_like_match_arms(
        &e.variants,
        &factory_state_enum_type,
        &factory_state_enum_type,
        &quote! { ref },
        |expr| quote! { ::core::clone::Clone::clone(#expr) },
    );
    let factory_to_self_transform = common::clone_like_match_arms(
        &e.variants,
        &factory_state_enum_type,
        ctx.item_type.path()?,
        &quote! {},
        |expr| quote! { #exhaust_crate_path::Exhaust::from_factory(#expr) },
    );

    let (impl_generics, ty_generics, augmented_where_predicates) =
        ctx.generics_with_bounds(syn::parse_quote! {});

    let impls = ctx.impl_iterator_and_factory_traits(
        quote! {
            match &mut self.0 {
                #( #variant_next_arms , )*
                #iter_state_enum_type::#done_variant => ::core::option::Option::None,
            }
        },
        quote! {
            Self(#first_state_variant_initializer)
        },
        quote! {
            Self(match &self.0 {
                #( #state_enum_variant_cloners , )*
            })
        },
    );

    let iterator_decl = quote! {
        // Struct that is exposed as the `<Self as Exhaust>::Iter` type.
        #vis struct #iterator_type_name #ty_generics
        (#iter_state_enum_type #ty_generics)
        where #augmented_where_predicates;

        // Struct that is exposed as the `<Self as Exhaust>::Factory` type.
        #vis struct #factory_outer_type_path #ty_generics (#factory_state_enum_type #ty_generics)
        where #augmented_where_predicates;

        #impls

        // Enum wrapped in #factory_type_name with the actual data.
        enum #factory_state_enum_type #ty_generics
        where #augmented_where_predicates { #( #factory_variant_decls ,)* }

        // A manual impl of Clone is required to *not* have a `Clone` bound on the generics.
        impl #impl_generics ::core::clone::Clone for #factory_type
        where #augmented_where_predicates {
            fn clone(&self) -> Self {
                #![allow(unreachable_code)] // in case of empty enum
                Self(match self.0 {
                    #( #factory_enum_variant_clone_arms , )*
                })
            }
        }

        enum #iter_state_enum_type #ty_generics
        where #augmented_where_predicates
        {
            #( #state_enum_variant_decls , )*
        }
    };

    let from_factory_body = quote! {
        match factory.0 {
            #( #factory_to_self_transform , )*
        }
    };

    Ok((iterator_decl, from_factory_body))
}
