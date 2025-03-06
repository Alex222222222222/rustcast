use std::collections::HashMap;

use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{Data, Generics};

pub fn struct_lazy_playlist_child(
    inner_name: &Ident,
    name: &Ident,
    generics: &Generics,
    data: &Data,
    input_types: &HashMap<String, syn::Type>,
) -> proc_macro2::TokenStream {
    let recursive = match data {
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
                    match name.to_string().as_str() {
                        "inner" => quote! {},
                        "played" => quote! {},
                        "byte_per_millisecond" => quote! {},
                        "current_index" => quote! {},
                        "title" => quote! {},
                        "artist" => quote! {},
                        "content_type" => quote! {},
                        "shuffle" => quote_spanned! {name.span()=>
                            #name: #f_type,
                        },
                        "repeat" => quote_spanned! {name.span()=>
                            #name: #f_type,
                        },
                        _ => quote_spanned! {name.span()=>
                            #name: Option<#f_type>,
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
    };

    quote! {
        // The generated struct.
        pub struct #name #generics {
            inner: Option<#inner_name>,
            #recursive
        }
    }
}
