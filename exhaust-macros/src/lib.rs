use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
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

fn exhaust_iter_struct(
    _s: syn::DataStruct,
    vis: syn::Visibility,
    target_type: Ident,
    iterator_ident: Ident,
) -> Result<TokenStream2, syn::Error> {
    let doc = iterator_doc(&iterator_ident);

    Ok(quote! {
        #[doc = #doc]
        #[derive(Clone, Debug, Default)]
        #vis struct #iterator_ident {
            // TODO: iterator state
        }

        impl ::core::iter::Iterator for #iterator_ident {
            type Item = #target_type;

            fn next(&mut self) -> Option<Self::Item> {
                todo!("struct exhaust iterator")
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
    let doc = iterator_doc(&iterator_ident);

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

fn iterator_doc(iterator_ident: &Ident) -> String {
    format!("Iterator over all values of [`{}`].", iterator_ident)
}
