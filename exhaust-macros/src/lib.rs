use itertools::Itertools as _;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens as _};
use syn::{parse_macro_input, DeriveInput};

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
        ident: target_type_ident,
        attrs: _,
        vis,
        generics: _, // TODO: process generics
        data,
    } = input;

    let iterator_ident = Ident::new(&format!("Exhaust{}", target_type_ident), Span::mixed_site());

    let iterator_implementation = match data {
        syn::Data::Struct(s) => {
            exhaust_iter_struct(s, vis, target_type_ident.clone(), iterator_ident.clone())
        }
        syn::Data::Enum(e) => {
            exhaust_iter_enum(e, vis, target_type_ident.clone(), iterator_ident.clone())
        }
        syn::Data::Union(syn::DataUnion { union_token, .. }) => Err(syn::Error::new(
            union_token.span,
            "derive(Exhaust) does not support unions",
        )),
    }?;

    Ok(quote! {
        impl ::exhaust::Exhaust for #target_type_ident {
            type Iter = #iterator_ident;
            fn exhaust() -> Self::Iter {
                ::core::default::Default::default()
            }
        }

        #iterator_implementation
    })
}

/// Given a set of fields to exhaust, generate fields and code for the iterator to
/// do that. This applies to structs and to enum variants.
///
/// This code generator cannot be used on zero fields; the caller should handle that
/// case, because that can be implemented more efficiently given knowledge of the case
/// where the type is an enum.
fn exhaust_iter_fields(
    struct_fields: &syn::Fields,
    constructor: TokenStream2,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    assert!(
        !struct_fields.is_empty(),
        "exhaust_iter_fields requires at least 1 field"
    );
    let (iterator_fields, iterator_fields_init, field_names, field_types): (
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
    ) = struct_fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let field_name = match &field.ident {
                Some(name) => name.to_token_stream(),
                None => quote! { #index },
            };
            let field_type = &field.ty;
            (
                quote! {
                    #field_name : ::core::iter::Peekable<
                        <#field_type as ::exhaust::Exhaust>::Iter
                    >
                },
                quote! {
                    #field_name : ::exhaust::iteration::peekable_exhaust::<#field_type>()
                },
                field_name,
                field_type.clone().to_token_stream(),
            )
        })
        .multiunzip();

    let field_value_getters = field_names.iter().enumerate().map(|(i, name)| {
        // unwrap() cannot fail because we checked with peek() before this code runs.
        // TODO: Can we manage to extract this pattern to a helper module?
        if i == field_names.len() - 1 {
            // Advance the "last digit".
            quote! { ::core::iter::Iterator::next(&mut self.#name).unwrap() }
        } else {
            // Don't advance the others
            quote! { ::core::clone::Clone::clone(::core::iter::Peekable::peek(&mut self.#name).unwrap()) }
        }
    });

    let carries = field_names
        .iter()
        .zip(field_names.iter().skip(1).zip(field_types.iter().skip(1)))
        .rev()
        .map(|(high, (low, low_field_type))| {
            quote! {
                ::exhaust::iteration::carry(
                    &mut self.#high,
                    &mut self.#low,
                    ::exhaust::iteration::peekable_exhaust::<#low_field_type>
                )
            }
        });

    // This implementation is analogous to exhaust::ExhaustArray, except that instead of
    // iterating over the indices it has to hardcode each one.
    let next_fn_implementation = quote! {
        // Check if we have a next item
        let has_next = #( self.#field_names.peek().is_some() && )* true;
        if !has_next {
            return None;
        }

        // Gather that next item, advancing the last field iterator.
        let item = #constructor { #( #field_names : #field_value_getters , )* };

        // Perform carries to other field iterators.
        #[allow(clippy::short_circuit_statement)]
        {
            let _ = #( #carries && )* true;
        }

        Some(item)
    };
    (
        quote! {
            #( #iterator_fields , )*
        },
        quote! {
            #( #iterator_fields_init , )*
        },
        next_fn_implementation,
    )
}

fn exhaust_iter_struct(
    s: syn::DataStruct,
    vis: syn::Visibility,
    target_type: Ident,
    iterator_ident: Ident,
) -> Result<TokenStream2, syn::Error> {
    let doc = iterator_doc(&target_type);
    let (iterator_fields, iterator_fields_init, iterator_code) = if s.fields.is_empty() {
        (
            quote! { done: bool, },
            quote! { done: false, },
            quote! {
                if self.done {
                    None
                } else {
                    self.done = true;
                    Some(#target_type {})
                }
            },
        )
    } else {
        exhaust_iter_fields(&s.fields, target_type.to_token_stream())
    };

    Ok(quote! {
        #[doc = #doc]
        #[derive(Clone, Debug)]
        #vis struct #iterator_ident {
            #iterator_fields
        }

        impl ::core::iter::Iterator for #iterator_ident {
            type Item = #target_type;

            fn next(&mut self) -> Option<Self::Item> {
                #iterator_code
            }
        }

        impl ::core::default::Default for #iterator_ident {
            fn default() -> Self {
                Self {
                    #iterator_fields_init
                }
            }
        }
    })
}

fn exhaust_iter_enum(
    e: syn::DataEnum,
    vis: syn::Visibility,
    target_type: Ident,
    iterator_ident: Ident,
) -> Result<TokenStream2, syn::Error> {
    let doc = iterator_doc(&target_type);

    // Generate a `Chain` of iterators. This is not an optimal implementation,
    // because it does extra work per step; ideally we would generate an enum parallel
    // to the original which produces
    let mut inner_iterator_type_and_initializer = None;
    for variant in e.variants.iter() {
        let variant_ident = &variant.ident;
        let variant_iterator_type = quote! { ::core::iter::Once<#target_type> };
        let variant_iterator_initializer =
            quote! { ::core::iter::once(#target_type :: #variant_ident {}) };
        inner_iterator_type_and_initializer = Some(
            if let Some((previous_type, previous_init)) = inner_iterator_type_and_initializer {
                (
                    quote! { ::core::iter::Chain<#previous_type, #variant_iterator_type> },
                    quote! { ::core::iter::Iterator::chain(#previous_init, #variant_iterator_initializer) },
                )
            } else {
                (variant_iterator_type, variant_iterator_initializer)
            },
        );
    }
    let (inner_iterator_type, inner_iterator_initializer) =
        match inner_iterator_type_and_initializer {
            Some((t, i)) => (t, i),
            None => (
                quote! { ::core::iter::Empty },
                quote! { ::core::iter::empty },
            ),
        };

    Ok(quote! {
        #[doc = #doc]
        #[derive(Clone, Debug)]
        #vis struct #iterator_ident(#inner_iterator_type);

        impl ::core::iter::Iterator for #iterator_ident {
            type Item = #target_type;

            fn next(&mut self) -> ::core::option::Option<Self::Item> {
                self.0.next()
            }
        }

        impl ::core::default::Default for #iterator_ident {
            fn default() -> Self {
                Self(#inner_iterator_initializer)
            }
        }
    })
}

fn iterator_doc(type_name: &Ident) -> String {
    format!("Iterator over all values of [`{}`].", type_name)
}
