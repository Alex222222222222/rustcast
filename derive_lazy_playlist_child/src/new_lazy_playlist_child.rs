use std::collections::HashMap;

use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{Data, Generics};

use crate::IGNORED_FIELDS;

pub fn new_lazy_playlist_child(
    name: &Ident,
    generics: &Generics,
    data: &Data,
    input_types: &HashMap<String, syn::Type>,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let input = new_lazy_playlist_child_input(data, input_types);
    let constructor = new_lazy_playlist_child_constructor(data);

    quote! {
        // The generated impl.
        impl #impl_generics #name #ty_generics #where_clause {
            pub async fn new(#input) -> anyhow::Result<Self> {
                Ok(#constructor)
            }
        }
    }
}

fn new_lazy_playlist_child_constructor(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = match &f.ident {
                        Some(name) => name,
                        None => panic!("Unnamed field is not supported"),
                    };
                    if IGNORED_FIELDS.contains(&name.to_string().as_str()) {
                        return quote! {};
                    }
                    match name.to_string().as_str() {
                        "shuffle" => quote_spanned! {name.span()=>
                            #name: #name.unwrap_or(false),
                        },
                        "repeat" => quote_spanned! {name.span()=>
                            #name: #name.unwrap_or(false),
                        },
                        _ => quote_spanned! {name.span()=>
                            #name: Some(#name),
                        },
                    }
                });
                quote! {
                    Self {
                        inner: None,
                        #(#recurse)*
                    }
                }
            }
            _ => panic!("Only named struct is supported"),
        },
        _ => panic!("Only struct is supported"),
    }
}

fn new_lazy_playlist_child_input(
    data: &Data,
    input_types: &HashMap<String, syn::Type>,
) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = match &f.ident {
                        Some(name) => name,
                        None => panic!("Unnamed field is not supported"),
                    };
                    let f_type = match input_types.get(&name.to_string()) {
                        Some(f_type) => f_type,
                        None => &f.ty,
                    };
                    if IGNORED_FIELDS.contains(&name.to_string().as_str()) {
                        return quote! {};
                    }
                    match name.to_string().as_str() {
                        "shuffle" => quote_spanned! {name.span()=>
                            #name: Option<#f_type>,
                        },
                        "repeat" => quote_spanned! {name.span()=>
                            #name: Option<#f_type>,
                        },
                        _ => quote_spanned! {name.span()=>
                            #name: #f_type,
                        },
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            _ => panic!("Only named struct is supported"),
        },
        _ => panic!("Only struct is supported"),
    }
}
