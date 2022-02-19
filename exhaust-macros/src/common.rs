use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
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
    pub item_type: ConstructorSyntax,

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
        match &self.item_type {
            // TODO: No tests to validate this doc link
            ConstructorSyntax::Braced(name) => format!(
                "Iterator over all values of [`{}`].\n\
                \n\
                To obtain an instance of this iterator, call [`Exhaust::exhaust()`].\n\
                \n\
                [`Exhaust::exhaust()`]: {}::Exhaust",
                name,
                self.exhaust_crate_path.to_token_stream()
            ),
            ConstructorSyntax::Tuple => {
                format!(
                    "Iterator over all tuples of {} elements.\n\
                    \n\
                    To obtain an instance of this iterator, call [`Exhaust::exhaust()`].",
                    self.generics.params.len()
                )
            }
        }
    }

    /// Generate the common parts of the Iterator implementation.
    pub fn impl_iterator_traits(
        &self,
        iterator_next_body: TokenStream2,
        iterator_default_body: TokenStream2,
    ) -> TokenStream2 {
        let iterator_type_name = &self.iterator_type_name;
        let (impl_generics, ty_generics, augmented_where_predicates) =
            self.generics_with_bounds(syn::parse_quote! {});
        let (_, _, debug_where_predicates) =
            self.generics_with_bounds(syn::parse_quote! { ::core::fmt::Debug });
        let item_type_inst = self.item_type.parameterized(&self.generics);

        quote! {
            impl #impl_generics ::core::iter::Iterator for #iterator_type_name #ty_generics
            where #augmented_where_predicates {
                type Item = #item_type_inst;

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

/// How to name a type for construction.
pub(crate) enum ConstructorSyntax {
    /// A struct or variant name to used with `MyStruct { field: value }` syntax.
    Braced(TokenStream2),
    /// The type is a primitive tuple.
    Tuple,
}

impl ConstructorSyntax {
    /// Name to use for concatenation to construct new type names.
    pub fn name_for_incorporation(&self) -> Result<String, syn::Error> {
        match self {
            ConstructorSyntax::Braced(name) => Ok(name.to_string()),
            ConstructorSyntax::Tuple => Err(syn::Error::new(
                Span::call_site(),
                "exhaust-macros internal error: no name for tuple types",
            )),
        }
    }

    /// Type applied to given type parameters.
    pub(crate) fn parameterized(&self, generics: &syn::Generics) -> TokenStream2 {
        match self {
            ConstructorSyntax::Braced(name) => {
                let (_, ty_generics, _) = generics.split_for_impl();
                quote! { #name #ty_generics }
            }
            ConstructorSyntax::Tuple => {
                let par = generics.type_params();
                quote! { ( #( #par , )* ) }
            }
        }
    }

    /// Constructor applied to fields.
    /// The fields MUST be in original declardd order, to handle the tuple case.
    pub(crate) fn value_expr<'a>(
        &self,
        names: impl Iterator<Item = &'a TokenStream2>,
        values: impl Iterator<Item = &'a TokenStream2>,
    ) -> TokenStream2 {
        match self {
            ConstructorSyntax::Braced(name) => {
                quote! { #name { #( #names : #values , )* } }
            }
            ConstructorSyntax::Tuple => {
                quote! { ( #( #values , )* ) }
            }
        }
    }

    /// Given an enum type name, produce a variant constructor.
    pub(crate) fn with_variant(&self, target_variant_ident: &Ident) -> ConstructorSyntax {
        match self {
            ConstructorSyntax::Braced(name) => {
                let mut name = name.clone();
                name.extend(quote! { :: });
                name.extend(target_variant_ident.to_token_stream());
                ConstructorSyntax::Braced(name)
            }
            ConstructorSyntax::Tuple => panic!("a tuple is not an enum"),
        }
    }
}
