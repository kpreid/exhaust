use std::iter;

use itertools::{izip, Itertools as _};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens as _};
use syn::spanned::Spanned;
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

/// Pieces of the implementation of a product iterator, over fields of a struct
/// or enum variant.
struct ExhaustFields {
    /// Field declarations for the iterator state, with trailing comma.
    field_decls: TokenStream2,
    /// Field initializers for [`Self::fields`], with trailing comma.
    initializers: TokenStream2,
    /// Patterns to bind the fields.
    field_pats: TokenStream2,
    /// Code to implement advancing the iterator. [`Self::field_pats`] should be in scope.
    advance: TokenStream2,
}

/// Given a set of fields to exhaust, generate fields and code for the iterator to
/// do that. This applies to structs and to enum variants.
///
/// This code generator cannot be used on zero fields; the caller should handle that
/// case, because that can be implemented more efficiently given knowledge of the case
/// where the type is an enum.
fn exhaust_iter_fields(struct_fields: &syn::Fields, constructor: TokenStream2) -> ExhaustFields {
    assert!(
        !struct_fields.is_empty(),
        "exhaust_iter_fields requires at least 1 field"
    );
    let (iterator_fields, iterator_fields_init, iter_field_names, target_field_names, field_types): (
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
                None => syn::LitInt::new(&format!("{}", index), Span::mixed_site()).to_token_stream(),
            };
            let iter_field_name = match &field.ident {
                Some(name) => name.to_token_stream(),
                None => Ident::new(&format!("f{}", index), Span::mixed_site()).to_token_stream(),
            };
            let field_type = &field.ty;
            (
                quote! {
                    #iter_field_name : ::core::iter::Peekable<
                        <#field_type as ::exhaust::Exhaust>::Iter
                    >
                },
                quote! {
                    #iter_field_name : ::exhaust::iteration::peekable_exhaust::<#field_type>()
                },
                iter_field_name,
                target_field_name,
                field_type.clone().to_token_stream(),
            )
        })
        .multiunzip();

    let field_value_getters = iter_field_names.iter().enumerate().map(|(i, name)| {
        // unwrap() cannot fail because we checked with peek() before this code runs.
        // TODO: Can we manage to extract this pattern to a helper module?
        if i == iter_field_names.len() - 1 {
            // Advance the "last digit".
            quote! { ::core::iter::Iterator::next(#name).unwrap() }
        } else {
            // Don't advance the others
            quote! { ::core::clone::Clone::clone(::core::iter::Peekable::peek(#name).unwrap()) }
        }
    });

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
                ::exhaust::iteration::carry(
                    #high,
                    #low,
                    ::exhaust::iteration::peekable_exhaust::<#low_field_type>
                )
            }
        });

    // This implementation is analogous to exhaust::ExhaustArray, except that instead of
    // iterating over the indices it has to hardcode each one.
    let next_fn_implementation = quote! {
        // Check if we have a next item
        // TODO: fix hygeine w.r.t pattern bound fields and local variables
        let has_next = #( #iter_field_names.peek().is_some() && )* true;
        if !has_next {
            return None;
        }

        // Gather that next item, advancing the last field iterator.
        let item = #constructor { #( #target_field_names : #field_value_getters , )* };

        // Perform carries to other field iterators.
        #[allow(clippy::short_circuit_statement)]
        {
            let _ = #( #carries && )* true;
        }

        Some(item)
    };
    ExhaustFields {
        field_decls: quote! {
            #( #iterator_fields , )*
        },
        initializers: quote! {
            #( #iterator_fields_init , )*
        },
        field_pats: quote! {
            #( #iter_field_names , )*
        },
        advance: next_fn_implementation,
    }
}

fn exhaust_iter_struct(
    s: syn::DataStruct,
    vis: syn::Visibility,
    target_type: Ident,
    iterator_ident: Ident,
) -> Result<TokenStream2, syn::Error> {
    let doc = iterator_doc(&target_type);
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
                    None
                } else {
                    *done = true;
                    Some(#target_type {})
                }
            },
        }
    } else {
        exhaust_iter_fields(&s.fields, target_type.to_token_stream())
    };

    Ok(quote! {
        #[doc = #doc]
        #[derive(Clone, Debug)]
        #vis struct #iterator_ident {
            #field_decls
        }

        impl ::core::iter::Iterator for #iterator_ident {
            type Item = #target_type;

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self { #field_pats } => {
                        #advance
                    }
                }
            }
        }

        impl ::core::default::Default for #iterator_ident {
            fn default() -> Self {
                Self {
                    #initializers
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

    // TODO: hide the declaration of this
    let state_enum_type = Ident::new(
        &format!("__ExhaustEnum_{}", target_type),
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
        state_enum_variant_idents,
        state_enum_variant_initializers,
        state_enum_field_pats,
        state_enum_variant_advancers,
    ): (
        Vec<TokenStream2>,
        Vec<Ident>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
        Vec<TokenStream2>,
    ) = e
        .variants
        .iter()
        .zip(state_enum_progress_variants.iter())
        .map(|(target_variant, state_ident)| {
            let ExhaustFields {
                field_decls: state_fields_decls,
                initializers: state_fields_init,
                field_pats,
                advance,
            } = if target_variant.fields.is_empty() {
                ExhaustFields {
                    field_decls: quote! {},
                    initializers: quote! {},
                    field_pats: quote! {},
                    advance: quote! {
                        todo!("code for fieldless enums missing")
                    },
                }
            } else {
                let target_variant_ident = &target_variant.ident;
                exhaust_iter_fields(
                    &target_variant.fields,
                    quote! { #target_type :: #target_variant_ident },
                )
            };

            (
                quote! {
                    #state_ident {
                        #state_fields_decls
                    }
                },
                state_ident.clone(),
                quote! {
                    #state_enum_type :: #state_ident { #state_fields_init }
                },
                field_pats,
                advance,
            )
        })
        .chain(iter::once((
            done_variant.to_token_stream(),
            done_variant.clone(),
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
                    Some(#target_type::#target_variant_ident {})
                }
            } else {
                quote! {
                    let maybe_variant = { #field_advancer };
                    match maybe_variant {
                        Some(v) => Some(v),
                        None => {
                            self.0 = #next_state_initializer;
                            self.next() // TODO: kludge
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

    Ok(quote! {
        #[doc = #doc]
        #[derive(Clone, Debug)]
        #vis struct #iterator_ident(#state_enum_type);

        impl ::core::iter::Iterator for #iterator_ident {
            type Item = #target_type;

            fn next(&mut self) -> ::core::option::Option<Self::Item> {
                match &mut self.0 {
                    #( #variant_next_arms , )*
                    #state_enum_type::#done_variant => None,
                }
            }
        }

        impl ::core::default::Default for #iterator_ident {
            fn default() -> Self {
                Self(#first_state_variant_initializer)
            }
        }

        #[derive(Clone, Debug)]
        enum #state_enum_type {
            #( #state_enum_variant_decls , )*
        }
    })
}

fn iterator_doc(type_name: &Ident) -> String {
    format!("Iterator over all values of [`{}`].", type_name)
}
