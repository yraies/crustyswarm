extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, IdentFragment};
use syn::{parse_macro_input, Data, DeriveInput, Ident, Index};

#[proc_macro_derive(Differentiable)]
pub fn derive_differentiable(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    differentiable(&ast).into()
}
fn differentiable(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let fq_trait = format_ident!("Differentiable");

    quote!(impl #fq_trait for #name {})
}

#[proc_macro_derive(AllOIDETraits)]
pub fn derive_all_oide_traits(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let add = add(&ast);
    let diff = diff(&ast);
    let scale = scale(&ast);
    let opposite = opposite(&ast);
    let randomize = randomize(&ast);
    let bound_application = bound_application(&ast);
    let differentiable = differentiable(&ast);

    quote!(
    #add
    #diff
    #scale
    #opposite
    #randomize
    #bound_application
    #differentiable
    )
    .into()
}

#[proc_macro_derive(OIDEAdd)]
pub fn derive_add(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    add(&ast).into()
}
fn add(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let fq_trait = format_ident!("OIDEAdd");

    let signature = quote!(fn add(&self, other0: &Self) -> Self);
    let self_params = 1;

    let named_struct_fn = Box::new(|id: &dyn IdentFragment| {
        let (s, o) = (format_ident!("{}", id), format_ident!("{}", id));
        quote!( ::add( &self.#s, &other0.#o ) )
    });
    let named_enum_fn = Box::new(|id: &dyn IdentFragment| {
        let (s, o) = (format_ident!("s_{}", id), format_ident!("o0_{}", id));
        quote!( ::add( &#s, &#o ))
    });
    let unnamed_fn = Box::new(|id: &Index| quote!( ::add( &self.#id, &other0.#id )));

    impl_semi_group_like_foo(
        &ast.ident,
        &ast.data,
        &fq_trait,
        signature,
        self_params,
        named_struct_fn,
        named_enum_fn,
        unnamed_fn,
    )
}

#[proc_macro_derive(OIDEDiff)]
pub fn derive_diff(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    diff(&ast).into()
}
fn diff(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let fq_trait = format_ident!("OIDEDiff");

    let signature = quote!(fn difference(&self, other0: &Self) -> Self);
    let self_params = 1;

    let named_struct_fn = Box::new(|id: &dyn IdentFragment| {
        let (s, o) = (format_ident!("{}", id), format_ident!("{}", id));
        quote!( ::difference( &self.#s, &other0.#o ) )
    });
    let named_enum_fn = Box::new(|id: &dyn IdentFragment| {
        let (s, o) = (format_ident!("s_{}", id), format_ident!("o0_{}", id));
        quote!( ::difference( &#s, &#o ))
    });
    let unnamed_fn = Box::new(|id: &Index| quote!( ::difference( &self.#id, &other0.#id )));

    impl_semi_group_like_foo(
        &ast.ident,
        &ast.data,
        &fq_trait,
        signature,
        self_params,
        named_struct_fn,
        named_enum_fn,
        unnamed_fn,
    )
}

#[proc_macro_derive(OIDEScale)]
pub fn derive_scale(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    scale(&ast).into()
}
fn scale(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let fq_trait = format_ident!("OIDEScale");

    let signature = quote!(fn scale(&self, factor: f32) -> Self);
    let self_params = 0;

    let named_struct_fn = Box::new(|id: &dyn IdentFragment| {
        let s = format_ident!("{}", id);
        quote!( ::scale( &self.#s, factor) )
    });
    let named_enum_fn = Box::new(|id: &dyn IdentFragment| {
        let s = format_ident!("s_{}", id);
        quote!( ::scale( &#s, factor ))
    });
    let unnamed_fn = Box::new(|id: &Index| quote!( ::scale( &self.#id, factor )));

    impl_semi_group_like_foo(
        &ast.ident,
        &ast.data,
        &fq_trait,
        signature,
        self_params,
        named_struct_fn,
        named_enum_fn,
        unnamed_fn,
    )
}

#[proc_macro_derive(OIDEOpposite)]
pub fn derive_opposite(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    opposite(&ast).into()
}
fn opposite(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let fq_trait = format_ident!("OIDEOpposite");

    let signature = quote!(fn opposite(&self) -> Self);
    let self_params = 0;

    let named_struct_fn = Box::new(|id: &dyn IdentFragment| {
        let s = format_ident!("{}", id);
        quote!( ::opposite( &self.#s ) )
    });
    let named_enum_fn = Box::new(|id: &dyn IdentFragment| {
        let s = format_ident!("s_{}", id);
        quote!( ::opposite( &#s ))
    });
    let unnamed_fn = Box::new(|id: &Index| quote!( ::opposite( &self.#id )));

    impl_semi_group_like_foo(
        &ast.ident,
        &ast.data,
        &fq_trait,
        signature,
        self_params,
        named_struct_fn,
        named_enum_fn,
        unnamed_fn,
    )
}

#[proc_macro_derive(OIDERandomize)]
pub fn derive_randomize(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    randomize(&ast).into()
}
fn randomize(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let fq_trait = format_ident!("OIDERandomize");

    let signature = quote!(fn random(&self, rng: &mut impl ::rand::Rng) -> Self);
    let self_params = 0;

    let named_struct_fn = Box::new(|id: &dyn IdentFragment| {
        let s = format_ident!("{}", id);
        quote!( ::random( &self.#s, rng ) )
    });
    let named_enum_fn = Box::new(|id: &dyn IdentFragment| {
        let s = format_ident!("s_{}", id);
        quote!( ::random( &#s, rng ))
    });
    let unnamed_fn = Box::new(|id: &Index| quote!( ::random( &self.#id, rng )));

    impl_semi_group_like_foo(
        &ast.ident,
        &ast.data,
        &fq_trait,
        signature,
        self_params,
        named_struct_fn,
        named_enum_fn,
        unnamed_fn,
    )
}

#[proc_macro_derive(OIDEBoundApplication)]
pub fn derive_bound_application(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    bound_application(&ast).into()
}
fn bound_application(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let fq_trait = format_ident!("OIDEBoundApplication");

    let signature = quote!(fn apply_bounds(&self, other0: &Self) -> Self);
    let self_params = 1;

    let named_struct_fn = Box::new(|id: &dyn IdentFragment| {
        let (s, o) = (format_ident!("{}", id), format_ident!("{}", id));
        quote!( ::apply_bounds( &self.#s, &other0.#o ) )
    });
    let named_enum_fn = Box::new(|id: &dyn IdentFragment| {
        let (s, o) = (format_ident!("s_{}", id), format_ident!("o0_{}", id));
        quote!( ::apply_bounds( &#s, &#o ))
    });
    let unnamed_fn = Box::new(|id: &Index| quote!( ::apply_bounds( &self.#id, &other0.#id )));

    impl_semi_group_like_foo(
        &ast.ident,
        &ast.data,
        &fq_trait,
        signature,
        self_params,
        named_struct_fn,
        named_enum_fn,
        unnamed_fn,
    )
}

fn impl_semi_group_like_foo(
    implementor: &Ident,
    data: &Data,
    trait_id: &Ident,
    signature: quote::__private::TokenStream,
    self_params: usize,
    named_struct_fn: Box<dyn Fn(&dyn IdentFragment) -> quote::__private::TokenStream>,
    enum_fn: Box<dyn Fn(&dyn IdentFragment) -> quote::__private::TokenStream>,
    unnamed_struct_fn: Box<dyn Fn(&Index) -> quote::__private::TokenStream>,
) -> proc_macro2::TokenStream {
    let contents = match data {
        Data::Struct(s) => match &s.fields {
            syn::Fields::Named(named) => {
                let fields: Vec<_> = named.named.iter().flat_map(|f| &f.ident).collect();

                let filllines = fields
                    .iter()
                    .map(|id| (id, named_struct_fn(id)))
                    .map(|(id, new_val)| quote!( #id: #trait_id#new_val));

                quote!(
                    #implementor { #(#filllines),* }
                )
            }
            syn::Fields::Unnamed(unnamed) => {
                let fields: Vec<_> = (0..unnamed.unnamed.len()).map(syn::Index::from).collect();

                let filllines = fields
                    .iter()
                    .map(|id| unnamed_struct_fn(id))
                    .map(|new_val| quote!(#trait_id#new_val));
                quote!(
                    #implementor ( #(#filllines),* )
                )
            }
            syn::Fields::Unit => quote!( #implementor ),
        },
        Data::Enum(e) => {
            let variant_arm_quotes = e.variants.iter().map(|v| {
                let vident = &v.ident;
                match &v.fields {
                    syn::Fields::Named(named) => {
                        let fields: Vec<_> = named.named.iter().flat_map(|f| &f.ident).collect();

                        let parent_ids: Vec<String> = vec!["s_".to_string()]
                            .into_iter()
                            .chain(
                                (0usize..self_params)
                                    .into_iter()
                                    .map(|idx| format!("o{}_", idx)),
                            )
                            .collect();

                        let lhs_blocks = parent_ids.iter().map(|prefix| {
                            let f = fields.iter().map(|field| {
                                let rs = format_ident!("{}{}", &prefix, &field);
                                quote!( #field: #rs )
                            });
                            quote!( #implementor::#vident { #( #f ),* } )
                        });

                        let filllines: Vec<_> = fields
                            .iter()
                            .map(|id| (id, enum_fn(id)))
                            .map(|(id, new_val)| quote!( #id: #trait_id#new_val ))
                            .collect();
                        let rhs = quote!( #implementor::#vident{ #(#filllines),* } );

                        quote!(( #(#lhs_blocks),* ) => #rhs)
                    }
                    syn::Fields::Unnamed(unnamed) => {
                        let indices: Vec<_> =
                            (0..unnamed.unnamed.len()).map(syn::Index::from).collect();

                        let parent_ids: Vec<String> = vec!["s_".to_string()]
                            .into_iter()
                            .chain(
                                (0usize..self_params)
                                    .into_iter()
                                    .map(|idx| format!("o{}_", idx)),
                            )
                            .collect();

                        let lhs_blocks = parent_ids.iter().map(|prefix| {
                            let f = indices.iter().map(|field| {
                                let rs = format_ident!("{}{}", &prefix, &field);
                                quote!( #rs )
                            });
                            quote!( #implementor::#vident ( #( #f ),* ) )
                        });

                        let filllines: Vec<_> = indices
                            .iter()
                            .map(|id| (id, enum_fn(id)))
                            .map(|(_id, new_val)| quote!( #trait_id#new_val ))
                            .collect();
                        let rhs = quote!( #implementor::#vident( #(#filllines),* ) );

                        quote!(( #(#lhs_blocks),* ) => #rhs)
                    }
                    syn::Fields::Unit => {
                        let lhs = (0..=self_params).map(|_| quote!(#implementor::#vident));
                        quote!(( #( #lhs ),* ) => #implementor::#vident)
                    }
                }
            });
            let params: Vec<_> = (0usize..self_params)
                .into_iter()
                .map(|idx| format_ident!("other{}", idx))
                .collect();
            quote!(
                match (self #(, #params)*) {
                    #(#variant_arm_quotes ,)*
                    _ => panic!("All Self parameters have to be the same enum variant!"),
                }
            )
        }
        Data::Union(_u) => {
            quote!()
        }
    };

    quote!(
        impl #trait_id for #implementor {
            #signature {
                #contents
            }
        }
    )
}
