use std::iter;

use itertools::{izip, Itertools as _};
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

/// Derive macro generating an impl of the trait `exhaust::Exhaust`.
///
/// This may be applied to `struct`s and `enum`s, but not `union`s.
///
/// The generated iterator type will have the name of the given type with `Exhaust`
/// prepended, and the same visibility.
///
/// TODO: Document what optional functionality the generated iterator has.
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

    let ctx = ExhaustContext {
        vis,
        generics,
        iterator_type_name: Ident::new(&format!("Exhaust{}", item_type_name), Span::mixed_site()),
        item_type: ConstructorSyntax::Braced(item_type_name.to_token_stream()),
        exhaust_crate_path: syn::parse_quote! { ::exhaust },
    };
    let ExhaustContext {
        iterator_type_name,
        exhaust_crate_path,
        ..
    } = &ctx;

    let iterator_implementation = match data {
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
        impl #impl_generics #exhaust_crate_path::Exhaust for #item_type_name #ty_generics
        where #augmented_where_predicates {
            type Iter = #iterator_type_name #ty_generics;
            fn exhaust() -> Self::Iter {
                ::core::default::Default::default()
            }
        }

        #iterator_implementation
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
        .map(|i| Ident::new(&format!("T{}", i), Span::mixed_site()))
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
        iterator_type_name: Ident::new(&format!("ExhaustTuple{}", size), Span::mixed_site()),
        exhaust_crate_path: parse_quote! { crate },
    };

    let iterator_type_name = &ctx.iterator_type_name;

    // Generate the field-exhausting iteration logic
    let ExhaustFields {
        field_decls,
        initializers,
        field_pats,
        advance,
    } = exhaust_iter_fields(&ctx, &synthetic_fields, &ConstructorSyntax::Tuple);

    let iterator_impls = ctx.impl_iterator_traits(
        quote! {
            match self {
                Self { #field_pats } => {
                    #advance
                }
            }
        },
        quote! { Self { #initializers } },
    );

    let iterator_doc = ctx.iterator_doc();

    Ok(quote! {
        impl<#( #value_type_vars , )*> crate::Exhaust for ( #( #value_type_vars , )* )
        where #( #value_type_vars : crate::Exhaust, )*
        {
            type Iter = #iterator_type_name <#( #value_type_vars , )*>;
            fn exhaust() -> Self::Iter {
                ::core::default::Default::default()
            }
        }

        #[doc = #iterator_doc]
        #[derive(Clone)]
        pub struct #iterator_type_name <#( #value_type_vars , )*>
        where #( #value_type_vars : crate::Exhaust, )*
        {
            #field_decls
        }

        #iterator_impls
    })
}

fn exhaust_iter_struct(
    s: syn::DataStruct,
    ctx: &ExhaustContext,
) -> Result<TokenStream2, syn::Error> {
    let doc = ctx.iterator_doc();
    let vis = &ctx.vis;
    let iterator_type_name = &ctx.iterator_type_name;

    let ExhaustFields {
        field_decls,
        initializers,
        field_pats,
        advance,
    } = if s.fields.is_empty() {
        let empty_ctor = ctx.item_type.value_expr([].iter(), [].iter());
        ExhaustFields {
            field_decls: quote! { done: bool, },
            initializers: quote! { done: false, },
            field_pats: quote! { done, },
            advance: quote! {
                if *done {
                    ::core::option::Option::None
                } else {
                    *done = true;
                    ::core::option::Option::Some(#empty_ctor)
                }
            },
        }
    } else {
        exhaust_iter_fields(ctx, &s.fields, &ctx.item_type)
    };

    let (_, ty_generics, augmented_where_predicates) =
        ctx.generics_with_bounds(syn::parse_quote! {});

    // Note: The iterator must have trait bounds because its fields, being of type
    // `<SomeOtherTy as Exhaust>::Iter`, require that `SomeOtherTy: Exhaust`.

    let impls = ctx.impl_iterator_traits(
        quote! {
            match self {
                Self { #field_pats } => {
                    #advance
                }
            }
        },
        quote! { Self { #initializers } },
    );

    Ok(quote! {
        #[doc = #doc]
        #[derive(Clone)]
        #vis struct #iterator_type_name #ty_generics
        where #augmented_where_predicates {
            #field_decls
        }

        #impls
    })
}

fn exhaust_iter_enum(e: syn::DataEnum, ctx: &ExhaustContext) -> Result<TokenStream2, syn::Error> {
    let doc = ctx.iterator_doc();
    let vis = &ctx.vis;
    let iterator_type_name = &ctx.iterator_type_name;

    // TODO: hide the declaration of this
    let state_enum_type = Ident::new(
        &format!("__ExhaustEnum_{}", ctx.item_type.name_for_incorporation()?),
        Span::mixed_site(),
    );

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
    let (
        state_enum_variant_decls,
        state_enum_variant_initializers,
        state_enum_field_pats,
        state_enum_variant_advancers,
    ): (
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
    ) = e
        .variants
        .iter()
        .zip(state_enum_progress_variants.iter())
        .map(|(target_variant, state_ident)| {
            let fields::ExhaustFields {
                field_decls: state_fields_decls,
                initializers: state_fields_init,
                field_pats,
                advance,
            } = if target_variant.fields.is_empty() {
                // TODO: don't even construct this dummy value (needs refactoring)
                fields::ExhaustFields {
                    field_decls: quote! {},
                    initializers: quote! {},
                    field_pats: quote! {},
                    advance: quote! {
                        compile_error!("can't happen: fieldless ExhaustFields not used")
                    },
                }
            } else {
                let target_variant_ident = &target_variant.ident;
                fields::exhaust_iter_fields(
                    ctx,
                    &target_variant.fields,
                    &ctx.item_type.with_variant(target_variant_ident),
                )
            };

            (
                quote! {
                    #state_ident {
                        #state_fields_decls
                    }
                },
                quote! {
                    #state_enum_type :: #state_ident { #state_fields_init }
                },
                field_pats,
                advance,
            )
        })
        .chain(iter::once((
            done_variant.to_token_stream(),
            quote! {
                #state_enum_type :: #done_variant {}
            },
            quote! {},
            quote! { compile_error!("done advancer not used") },
        )))
        .multiunzip();

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
                let empty_ctor = ctx
                    .item_type
                    .with_variant(target_variant_ident)
                    .value_expr([].iter(), [].iter());
                quote! {
                    self.0 = #next_state_initializer;
                    ::core::option::Option::Some(#empty_ctor)
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
                #state_enum_type::#state_ident { #pats } => {
                    #advancer
                }
            }
        },
    );

    let (_, ty_generics, augmented_where_predicates) =
        ctx.generics_with_bounds(syn::parse_quote! {});

    let impls = ctx.impl_iterator_traits(
        quote! {
            match &mut self.0 {
                #( #variant_next_arms , )*
                #state_enum_type::#done_variant => ::core::option::Option::None,
            }
        },
        quote! {
            Self(#first_state_variant_initializer)
        },
    );

    Ok(quote! {
        #[doc = #doc]
        #[derive(Clone)]
        #vis struct #iterator_type_name #ty_generics
        (#state_enum_type #ty_generics)
        where #augmented_where_predicates;

        #impls

        #[derive(Clone)]
        enum #state_enum_type #ty_generics
        where #augmented_where_predicates
        {
            #( #state_enum_variant_decls , )*
        }
    })
}
