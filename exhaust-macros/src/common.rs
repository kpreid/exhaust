use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens as _};
use syn::parse_quote;
use syn::punctuated::Punctuated;

/// Data and helpers for generating an exhaustive iterator, that are common
/// between the enum and struct versions.
pub(crate) struct ExhaustContext {
    /// Maximum visibility for the generated types, inherited from the declaration
    /// (e.g. `pub Foo` should have a `pub ExhaustFoo` iterator).
    pub vis: syn::Visibility,

    /// Generics present on the declaration, which need to be copied to the
    /// iterator.
    pub generics: syn::Generics,

    /// Name of the type being iterated.
    pub item_type_name: Ident,

    /// Name of the generated iterator type.
    pub iterator_type_name: Ident,

    /// Path by which the `exhaust` crate should be referred to.
    pub exhaust_crate_path: syn::Path,
}

impl ExhaustContext {
    /// Generate the TraitBound `exhaust::Exhaust`.
    pub fn exhaust_trait_bound(&self) -> syn::TraitBound {
        let mut path = self.exhaust_crate_path.clone();
        path.segments.push(parse_quote! { Exhaust });
        // reinterpret as TraitBound
        parse_quote! { #path }
    }

    /// As [`syn::Generics::split_for_impl`], but adding the given bounds,
    /// as well as the `::exhaust::Exhaust` bound.
    pub fn generics_with_bounds(
        &self,
        mut bounds: Punctuated<syn::TypeParamBound, syn::Token![+]>,
    ) -> (
        syn::ImplGenerics<'_>,
        syn::TypeGenerics<'_>,
        Punctuated<syn::WherePredicate, syn::token::Comma>,
    ) {
        bounds.push(syn::TypeParamBound::Trait(self.exhaust_trait_bound()));
        let generics = &self.generics;
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let mut augmented_where_predicates = match where_clause {
            Some(clause) => clause.predicates.clone(),
            None => Punctuated::new(),
        };
        for g in generics.params.iter() {
            if let syn::GenericParam::Type(g) = g {
                augmented_where_predicates.push(syn::WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::Type::Verbatim(g.ident.to_token_stream()),
                    colon_token: <_>::default(),
                    bounds: bounds.clone(),
                }));
            }
        }
        (impl_generics, ty_generics, augmented_where_predicates)
    }

    pub fn iterator_doc(&self) -> String {
        format!("Iterator over all values of [`{}`].", self.item_type_name)
    }

    /// Generate the common parts of the Iterator implementation.
    pub fn impl_iterator_traits(
        &self,
        iterator_next_body: TokenStream2,
        iterator_default_body: TokenStream2,
    ) -> TokenStream2 {
        let item_type_name = &self.item_type_name;
        let iterator_type_name = &self.iterator_type_name;
        let (impl_generics, ty_generics, augmented_where_predicates) =
            self.generics_with_bounds(syn::parse_quote! {});
        let (_, _, debug_where_predicates) =
            self.generics_with_bounds(syn::parse_quote! { ::core::fmt::Debug });

        quote! {
            impl #impl_generics ::core::iter::Iterator for #iterator_type_name #ty_generics
            where #augmented_where_predicates {
                type Item = #item_type_name #ty_generics;

                fn next(&mut self) -> ::core::option::Option<Self::Item> {
                    #iterator_next_body
                }
            }

            impl #impl_generics ::core::default::Default for #iterator_type_name #ty_generics
            where #augmented_where_predicates {
                fn default() -> Self {
                    #iterator_default_body
                }
            }

            // A manual impl of Debug would be required to provide the right bounds on the generics,
            // and given that we're implementing anyway, we might as well provide a cleaner format.
            impl #impl_generics ::core::fmt::Debug for #iterator_type_name #ty_generics
            where #debug_where_predicates {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    // TODO: print state
                    f.debug_struct(stringify!(#iterator_type_name))
                        .finish_non_exhaustive()
                }
            }
        }
    }
}
