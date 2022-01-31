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
        syn::Data::Struct(s) => exhaust_iter_struct(s, vis, iterator_ident.clone()),
        syn::Data::Enum(e) => exhaust_iter_enum(e, vis, iterator_ident.clone()),
        syn::Data::Union(syn::DataUnion { union_token, .. }) => Err(syn::Error::new(
            union_token.span,
            "derive(Exhaust) does not support unions",
        )),
    }?;

    Ok(quote! {
        impl ::exhaust::Exhaust for #target_type_ident {
            type Iter = #iterator_ident;
            fn exhaust() -> Self::Iter {
                #iterator_ident::new()
            }
        }

        #iterator_implementation
    })
}

fn exhaust_iter_struct(
    _s: syn::DataStruct,
    vis: syn::Visibility,
    iterator_ident: Ident,
) -> Result<TokenStream2, syn::Error> {
    Ok(quote! {
        #vis struct #iterator_ident {
            // TODO: iterator state
        }

        impl ::core::iter::Iterator for #iterator_ident {
            fn next(&mut self) -> Option<Self::Item> {
                todo!()
            }
        }
    })
}

fn exhaust_iter_enum(
    _e: syn::DataEnum,
    vis: syn::Visibility,
    iterator_ident: Ident,
) -> Result<TokenStream2, syn::Error> {
    Ok(quote! {
        #vis struct #iterator_ident {
            // TODO: iterator state
        }

        impl ::core::iter::Iterator for #iterator_ident {
            fn next(&mut self) -> Option<Self::Item> {
                todo!()
            }
        }
    })
}
