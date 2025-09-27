use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens as _};
use syn::punctuated::Punctuated;

use crate::common::{ConstructorSyntax, ExhaustContext};

/// Pieces of the implementation of a product iterator, over fields of a struct
/// or enum variant.
pub(crate) struct ExhaustFields {
    /// Field declarations for the iterator state struct/variant.
    pub state_field_decls: syn::Fields,
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
pub(crate) fn exhaustion_of_fields(
    ctx: &ExhaustContext,
    struct_fields: &syn::Fields,
    factory_outer_type_path: Option<&TokenStream2>,
    factory_inner_type_constructor: &ConstructorSyntax,
) -> ExhaustFields {
    assert!(
        !struct_fields.is_empty(),
        "exhaustion_of_fields requires at least 1 field"
    );

    let crate_path = &ctx.exhaust_crate_path;
    let helpers = ctx.helpers();

    #[allow(clippy::type_complexity)]
    let (
        iterator_state_fields,
        iterator_fields_init,
        iterator_fields_clone,
        iter_field_names,
        target_field_names,
        field_types,
        factory_value_vars,
    ): (
        Punctuated<syn::Field, syn::Token![,]>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
    ) = itertools::multiunzip(struct_fields.iter().enumerate().map(|(index, field)| {
        let target_field_name = match &field.ident {
            Some(name) => name.to_token_stream(),
            None => syn::LitInt::new(&format!("{index}"), Span::mixed_site()).to_token_stream(),
        };

        // Generate a field name to use in the iterator. By renaming the fields we ensure
        // they won't conflict with variables used in the rest of the iterator code.
        let iter_field_name = Ident::new(
            &match &field.ident {
                Some(name) => format!("iter_f_{name}"),
                None => format!("iter_f_{index}"),
            },
            Span::mixed_site(),
        );

        // Generate a variable name to use when fetching the current values of the iterators.
        let factory_var_name = Ident::new(
            &match &field.ident {
                Some(name) => format!("factory_{name}"),
                None => format!("factory_{index}"),
            },
            Span::mixed_site(),
        )
        .to_token_stream();

        let field_type = &field.ty;

        (
            syn::Field {
                attrs: Vec::new(),
                vis: syn::Visibility::Inherited,
                mutability: syn::FieldMutability::None,
                ident: Some(iter_field_name.clone()),
                colon_token: None,
                ty: syn::parse_quote! { #crate_path::iteration::Pei<#field_type> },
            },
            quote! {
                #iter_field_name : #crate_path::iteration::peekable_exhaust::<#field_type>()
            },
            quote! {
                #iter_field_name : #helpers::clone(#iter_field_name)
            },
            iter_field_name.to_token_stream(),
            target_field_name,
            field_type.clone().to_token_stream(),
            factory_var_name,
        )
    }));

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

    let state_field_decls = syn::Fields::Named(syn::FieldsNamed {
        brace_token: syn::token::Brace::default(),
        named: iterator_state_fields,
    });

    // Peek each field's iterator, except the last which is advanced.
    // The results of `field_iter_fetchers` will be matched against `Some(#factory_var)`.
    let (field_iter_fetchers, field_factory_exprs): (Vec<TokenStream2>, Vec<TokenStream2>) =
        iter_field_names
            .iter()
            .zip(factory_value_vars.iter())
            .enumerate()
            .map(|(i, (field_name, factory_var))| {
                // unwrap() cannot fail because we checked with peek() before this code runs.
                // TODO: Can we fit more of this in a non-macro helper?
                if i == iter_field_names.len() - 1 {
                    // Advance the "last digit".
                    (
                        quote! { #helpers::next(#field_name) },
                        factory_var.to_token_stream(),
                    )
                } else {
                    // Don't advance the others
                    (
                        quote! { #helpers::peek(#field_name) },
                        // Clone the peeked reference to get a factory value
                        quote! { #helpers::clone(#factory_var) },
                    )
                }
            })
            .unzip();

    let carries_expr = iter_field_names
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
        })
        // && short circuiting gives us the behavior we want conveniently, whereas
        // the nearest alternative would be to define a separate function.
        .collect::<Punctuated<TokenStream2, syn::Token![&&]>>();
    let carries_statement = match carries_expr.len() {
        0 => quote! {},
        // one function call
        1 => quote! { #carries_expr; },
        // explicitly ignore the unused final bool result of the `&&` expression
        2.. => quote! { let _ = #carries_expr; },
    };

    let factory_state_construction_expr = factory_inner_type_constructor
        .value_expr(target_field_names.iter(), field_factory_exprs.iter());

    let factory_construction_expr = if let Some(factory_outer_type_path) = factory_outer_type_path {
        quote! { #factory_outer_type_path(#factory_state_construction_expr) }
    } else {
        factory_state_construction_expr
    };

    // This implementation is analogous to exhaust::ExhaustArray, except that instead of
    // iterating over the indices it has to hardcode each one.
    let advance = quote! {
        // Gather factory values, peeking all but the last field and advancing the last field.
        if let (
            #( #helpers::Some(#factory_value_vars), )*
        ) = (#( #field_iter_fetchers, )*) {
            // Construct factory from its fields’ factories, cloning as needed.
            let factory = #factory_construction_expr;

            // Perform carries from any now-exhausted field iterators.
            #carries_statement

            #helpers::Some(factory)
        } else {
            #helpers::None
        }
    };
    ExhaustFields {
        state_field_decls,
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
