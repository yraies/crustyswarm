extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, IdentFragment};
use syn::{parse_macro_input, Data, DeriveInput, Index};

#[proc_macro_derive(Differentiable)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fq_trait = format_ident!("Differentiable");

    let mut functions: Vec<(
        quote::__private::TokenStream,
        usize,
        Box<dyn Fn(&dyn IdentFragment) -> quote::__private::TokenStream>,
        Box<dyn Fn(&dyn IdentFragment) -> quote::__private::TokenStream>,
        Box<dyn Fn(&Index) -> quote::__private::TokenStream>,
    )> = vec![];
    {
        functions.push((
            quote!(fn add(&self, other0: &Self) -> Self),
            1,
            Box::new(|id: &dyn IdentFragment| {
                let (s, o) = (format_ident!("{}", id), format_ident!("{}", id));
                quote!( #fq_trait::add( &self.#s, &other0.#o ) )
            }),
            Box::new(|id: &dyn IdentFragment| {
                let (s, o) = (format_ident!("s_{}", id), format_ident!("o0_{}", id));
                quote!( #fq_trait::add( &#s, &#o ))
            }),
            Box::new(|id: &Index| quote!( #fq_trait::add( &self.#id, &other0.#id ))),
        ));

        functions.push((
            quote!(fn difference(&self, other0: &Self) -> Self),
            1,
            Box::new(|id: &dyn IdentFragment| {
                let (s, o) = (format_ident!("{}", id), format_ident!("{}", id));
                quote!( #fq_trait::difference( &self.#s, &other0.#o ) )
            }),
            Box::new(|id: &dyn IdentFragment| {
                let (s, o) = (format_ident!("s_{}", id), format_ident!("o0_{}", id));
                quote!( #fq_trait::difference( &#s, &#o ))
            }),
            Box::new(|id: &Index| quote!( #fq_trait::difference( &self.#id, &other0.#id ))),
        ));

        functions.push((
            quote!(fn scale(&self, factor: f32) -> Self),
            0,
            Box::new(|id: &dyn IdentFragment| {
                let s = format_ident!("{}", id);
                quote!( #fq_trait::scale( &self.#s, factor) )
            }),
            Box::new(|id: &dyn IdentFragment| {
                let s = format_ident!("s_{}", id);
                quote!( #fq_trait::scale( &#s, factor ))
            }),
            Box::new(|id: &Index| quote!( #fq_trait::scale( &self.#id, factor ))),
        ));

        functions.push((
            quote!(fn opposite(&self) -> Self),
            0,
            Box::new(|id: &dyn IdentFragment| {
                let s = format_ident!("{}", id);
                quote!( #fq_trait::opposite( &self.#s ) )
            }),
            Box::new(|id: &dyn IdentFragment| {
                let s = format_ident!("s_{}", id);
                quote!( #fq_trait::opposite( &#s ))
            }),
            Box::new(|id: &Index| quote!( #fq_trait::opposite( &self.#id ))),
        ));

        functions.push((
            quote!(fn clon(&self) -> Self),
            0,
            Box::new(|id: &dyn IdentFragment| {
                let s = format_ident!("{}", id);
                quote!( #fq_trait::clon( &self.#s ) )
            }),
            Box::new(|id: &dyn IdentFragment| {
                let s = format_ident!("s_{}", id);
                quote!( #fq_trait::clon( &#s ))
            }),
            Box::new(|id: &Index| quote!( #fq_trait::clon( &self.#id ))),
        ));

        functions.push((
            quote!(fn random(&self, rng: &mut impl ::rand::Rng) -> Self),
            0,
            Box::new(|id: &dyn IdentFragment| {
                let s = format_ident!("{}", id);
                quote!( #fq_trait::random( &self.#s, rng ) )
            }),
            Box::new(|id: &dyn IdentFragment| {
                let s = format_ident!("s_{}", id);
                quote!( #fq_trait::random( &#s, rng ))
            }),
            Box::new(|id: &Index| quote!( #fq_trait::random( &self.#id, rng ))),
        ));

        functions.push((
            quote!(fn apply_bounds(&self, other0: &Self) -> Self),
            1,
            Box::new(|id: &dyn IdentFragment| {
                let (s, o) = (format_ident!("{}", id), format_ident!("{}", id));
                quote!( #fq_trait::apply_bounds( &self.#s, &other0.#o ) )
            }),
            Box::new(|id: &dyn IdentFragment| {
                let (s, o) = (format_ident!("s_{}", id), format_ident!("o0_{}", id));
                quote!( #fq_trait::apply_bounds( &#s, &#o ))
            }),
            Box::new(|id: &Index| quote!( #fq_trait::apply_bounds( &self.#id, &other0.#id ))),
        ));
    }

    let functionblocks: Vec<_> = match &ast.data {
        Data::Struct(s) => match &s.fields {
            syn::Fields::Named(named) => {
                let fields: Vec<_> = named.named.iter().flat_map(|f| &f.ident).collect();

                functions
                    .iter()
                    .map(|(signature, _, apply_s, _, _)| {
                        let filllines = fields
                            .iter()
                            .map(|id| (id, apply_s(id)))
                            .map(|(id, new_val)| quote!( #id: #new_val));

                        quote!(
                        #signature {
                            #name { #(#filllines),* }
                        }
                        )
                    })
                    .collect()
            }
            syn::Fields::Unnamed(unnamed) => {
                let fields: Vec<_> = (0..unnamed.unnamed.len()).map(syn::Index::from).collect();

                functions
                    .iter()
                    .map(|(signature, _, _, _, apply_u)| {
                        let filllines = fields
                            .iter()
                            .map(|id| apply_u(id))
                            .map(|new_val| quote!(#new_val));
                        quote!(
                        #signature {
                            #name ( #(#filllines),* )
                        }
                        )
                    })
                    .collect()
            }
            syn::Fields::Unit => functions
                .iter()
                .map(|(signature, _, _, _, _)| quote!( #signature { #name } ))
                .collect(),
        },
        Data::Enum(e) => functions
            .iter()
            .map(|(signature, extra_params, _, apply_e, _)| {
                let variant_arm_quotes = e.variants.iter().map(|v| {
                    let vident = &v.ident;
                    match &v.fields {
                        syn::Fields::Named(named) => {
                            let fields: Vec<_> =
                                named.named.iter().flat_map(|f| &f.ident).collect();

                            let parent_ids: Vec<String> = vec!["s_".to_string()]
                                .into_iter()
                                .chain(
                                    (0usize..*extra_params)
                                        .into_iter()
                                        .map(|idx| format!("o{}_", idx)),
                                )
                                .collect();

                            let lhs_blocks = parent_ids.iter().map(|prefix| {
                                let f = fields.iter().map(|field| {
                                    let rs = format_ident!("{}{}", &prefix, &field);
                                    quote!( #field: #rs )
                                });
                                quote!( #name::#vident { #( #f ),* } )
                            });

                            let filllines: Vec<_> = fields
                                .iter()
                                .map(|id| (id, apply_e(id)))
                                .map(|(id, new_val)| quote!( #id: #new_val ))
                                .collect();
                            let rhs = quote!( #name::#vident{ #(#filllines),* } );

                            quote!(( #(#lhs_blocks),* ) => #rhs)
                        }
                        syn::Fields::Unnamed(unnamed) => {
                            let indices: Vec<_> =
                                (0..unnamed.unnamed.len()).map(syn::Index::from).collect();

                            let parent_ids: Vec<String> = vec!["s_".to_string()]
                                .into_iter()
                                .chain(
                                    (0usize..*extra_params)
                                        .into_iter()
                                        .map(|idx| format!("o{}_", idx)),
                                )
                                .collect();

                            let lhs_blocks = parent_ids.iter().map(|prefix| {
                                let f = indices.iter().map(|field| {
                                    let rs = format_ident!("{}{}", &prefix, &field);
                                    quote!( #rs )
                                });
                                quote!( #name::#vident ( #( #f ),* ) )
                            });

                            let filllines: Vec<_> = indices
                                .iter()
                                .map(|id| (id, apply_e(id)))
                                .map(|(_id, new_val)| quote!( #new_val ))
                                .collect();
                            let rhs = quote!( #name::#vident( #(#filllines),* ) );

                            quote!(( #(#lhs_blocks),* ) => #rhs)
                        }
                        syn::Fields::Unit => {
                            let lhs = (0..extra_params + 1).map(|_| quote!(#name::#vident));
                            quote!(( #( #lhs ),* ) => #name::#vident)
                        }
                    }
                });
                let params: Vec<_> = (0usize..*extra_params)
                    .into_iter()
                    .map(|idx| format_ident!("other{}", idx))
                    .collect();
                quote!(
                #signature {
                    match (self #(, #params)*) {
                        #(#variant_arm_quotes ,)*
                        _ => panic!("All Self parameters have to be the same enum variant!"),
                    }
                })
            })
            .collect(),
        Data::Union(_u) => {
            vec![quote!()]
        }
    };

    let expanded = quote!(
    impl #fq_trait for #name {
        #(#functionblocks)*
    }
    );

    expanded.into()
}
