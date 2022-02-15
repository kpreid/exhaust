use std::iter;

use itertools::{izip, Itertools as _};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens as _};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput};

mod common;
use common::ExhaustContext;

mod fields;
use fields::{exhaust_iter_fields, ExhaustFields};

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
        item_type_name,
    };
    let ExhaustContext {
        item_type_name,
        iterator_type_name,
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
        ctx.generics_with_bounds(syn::parse_quote! { ::exhaust::Exhaust });

    Ok(quote! {
        impl #impl_generics ::exhaust::Exhaust for #item_type_name #ty_generics
        where #augmented_where_predicates {
            type Iter = #iterator_type_name #ty_generics;
            fn exhaust() -> Self::Iter {
                ::core::default::Default::default()
            }
        }

        #iterator_implementation
    })
}

fn exhaust_iter_struct(
    s: syn::DataStruct,
    ctx: &ExhaustContext,
) -> Result<TokenStream2, syn::Error> {
    let doc = ctx.iterator_doc();
    let vis = &ctx.vis;
    let item_type_name = &ctx.item_type_name;
    let iterator_type_name = &ctx.iterator_type_name;

    let ExhaustFields {
        field_decls,
        initializers,
        field_pats,
        advance,
    } = if s.fields.is_empty() {
        ExhaustFields {
            field_decls: quote! { done: bool, },
            initializers: quote! { done: false, },
            field_pats: quote! { done, },
            advance: quote! {
                if *done {
                    ::core::option::Option::None
                } else {
                    *done = true;
                    ::core::option::Option::Some(#item_type_name {})
                }
            },
        }
    } else {
        exhaust_iter_fields(&s.fields, item_type_name.to_token_stream())
    };

    let (_, ty_generics, augmented_where_predicates) =
        ctx.generics_with_bounds(syn::parse_quote! { ::exhaust::Exhaust });

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
    let item_type_name = &ctx.item_type_name;
    let iterator_type_name = &ctx.iterator_type_name;

    // TODO: hide the declaration of this
    let state_enum_type = Ident::new(
        &format!("__ExhaustEnum_{}", item_type_name),
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
                    &target_variant.fields,
                    quote! { #item_type_name :: #target_variant_ident },
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
                quote! {
                    self.0 = #next_state_initializer;
                    ::core::option::Option::Some(#item_type_name::#target_variant_ident {})
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
        ctx.generics_with_bounds(syn::parse_quote! { ::exhaust::Exhaust });

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
