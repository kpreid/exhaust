use itertools::Itertools as _;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens as _};

use crate::common::{ConstructorSyntax, ExhaustContext};

/// Pieces of the implementation of a product iterator, over fields of a struct
/// or enum variant.
pub(crate) struct ExhaustFields {
    /// Field declarations for the iterator state, with trailing comma.
    pub state_field_decls: TokenStream2,
    /// Field declarations for the factory struct/variant.
    pub factory_field_decls: syn::Fields,
    /// Field initializers for [`Self::fields`], with trailing comma.
    pub initializers: TokenStream2,
    /// Field cloning expressions for [`Self::fields`], with trailing comma.
    pub cloners: TokenStream2,
    /// Patterns to bind the fields.
    pub field_pats: TokenStream2,
    /// Code to implement advancing the iterator. [`Self::field_pats`] should be in scope.
    pub advance: TokenStream2,
}

/// Given a set of fields to exhaust, generate fields and code for the iterator to
/// do that. This applies to structs and to enum variants.
///
/// This code generator cannot be used on zero fields; the caller should handle that
/// case, because that can be implemented more efficiently given knowledge of the case
/// where the type is an enum.
pub(crate) fn exhaust_iter_fields(
    ctx: &ExhaustContext,
    struct_fields: &syn::Fields,
    factory_outer_type_path: &TokenStream2,
    factory_inner_type_constructor: &ConstructorSyntax,
) -> ExhaustFields {
    assert!(
        !struct_fields.is_empty(),
        "exhaust_iter_fields requires at least 1 field"
    );

    let crate_path = &ctx.exhaust_crate_path;

    #[allow(clippy::type_complexity)]
    let (
        iterator_fields,
        iterator_fields_init,
        iterator_fields_clone,
        iter_field_names,
        target_field_names,
        field_types,
    ): (
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
    ) = struct_fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let target_field_name = match &field.ident {
                Some(name) => name.to_token_stream(),
                None => {
                    syn::LitInt::new(&format!("{}", index), Span::mixed_site()).to_token_stream()
                }
            };

            // Generate a field name to use in the iterator. By renaming the fields we ensure
            // they won't conflict with variables used in the rest of the iterator code.
            let iter_field_name = Ident::new(
                &match &field.ident {
                    Some(name) => format!("iter_f_{}", name),
                    None => format!("iter_f_{}", index),
                },
                Span::mixed_site(),
            )
            .to_token_stream();

            let field_type = &field.ty;

            (
                quote! {
                    #iter_field_name : #crate_path::iteration::Pei<#field_type>
                },
                quote! {
                    #iter_field_name : #crate_path::iteration::peekable_exhaust::<#field_type>()
                },
                quote! {
                    #iter_field_name : ::core::clone::Clone::clone(#iter_field_name)
                },
                iter_field_name,
                target_field_name,
                field_type.clone().to_token_stream(),
            )
        })
        .multiunzip();

    let factory_field_decls = match struct_fields {
        syn::Fields::Named(fields) => syn::Fields::Named(syn::FieldsNamed {
            brace_token: syn::token::Brace::default(),
            named: fields
                .named
                .iter()
                .map(|field| -> syn::Field {
                    let ident = &field.ident;
                    let ty = &field.ty;
                    syn::parse_quote! {
                        #ident : <#ty as #crate_path::Exhaust>::Factory
                    }
                })
                .collect(),
        }),
        syn::Fields::Unnamed(fields) => syn::Fields::Unnamed(syn::FieldsUnnamed {
            paren_token: syn::token::Paren::default(),
            unnamed: fields
                .unnamed
                .iter()
                .map(|field| -> syn::Field {
                    let ty = &field.ty;
                    syn::parse_quote! {
                        <#ty as #crate_path::Exhaust>::Factory
                    }
                })
                .collect(),
        }),
        syn::Fields::Unit => syn::Fields::Unit,
    };

    let field_value_getters: Vec<TokenStream2> = iter_field_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            // unwrap() cannot fail because we checked with peek() before this code runs.
            // TODO: Can we fit more of this in a non-macro helper?
            if i == iter_field_names.len() - 1 {
                // Advance the "last digit".
                quote! { ::core::iter::Iterator::next(#name).unwrap() }
            } else {
                // Don't advance the others
                quote! { ::core::clone::Clone::clone(::core::iter::Peekable::peek(#name).unwrap()) }
            }
        })
        .collect();

    let carries = iter_field_names
        .iter()
        .zip(
            iter_field_names
                .iter()
                .skip(1)
                .zip(field_types.iter().skip(1)),
        )
        .rev()
        .map(|(high, (low, low_field_type))| {
            quote! {
                #crate_path::iteration::carry(
                    #high,
                    #low,
                    #crate_path::iteration::peekable_exhaust::<#low_field_type>
                )
            }
        });

    let factory_item_expr = factory_inner_type_constructor
        .value_expr(target_field_names.iter(), field_value_getters.iter());

    // This implementation is analogous to exhaust::ExhaustArray, except that instead of
    // iterating over the indices it has to hardcode each one.
    let advance = quote! {
        if #( #iter_field_names.peek().is_some() && )* true {
            // Gather that next factory, advancing the last field iterator only.
            let factory = #factory_outer_type_path(#factory_item_expr);

            // Perform carries to other field iterators.
            // && short circuiting gives us the behavior we want conveniently, whereas
            // the nearest alternative would be to define a separate function.
            let _ = #( #carries && )* true;

            ::core::option::Option::Some(factory)
        } else {
            ::core::option::Option::None
        }
    };
    ExhaustFields {
        state_field_decls: quote! {
            #( #iterator_fields , )*
        },
        factory_field_decls,
        initializers: quote! {
            #( #iterator_fields_init , )*
        },
        cloners: quote! {
            #( #iterator_fields_clone , )*
        },
        field_pats: quote! {
            #( #iter_field_names , )*
        },
        advance,
    }
}
