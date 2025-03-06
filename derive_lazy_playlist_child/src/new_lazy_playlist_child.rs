use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{Data, Generics};

pub fn new_lazy_playlist_child(
    name: &Ident,
    generics: &Generics,
    data: &Data,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let input = new_lazy_playlist_child_input(data);
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
                    match name.to_string().as_str() {
                        "inner" => quote! {},
                        "played" => quote! {},
                        "byte_per_millisecond" => quote! {},
                        "title" => quote! {},
                        "artist" => quote! {},
                        "content_type" => quote! {},
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

fn new_lazy_playlist_child_input(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = match &f.ident {
                        Some(name) => name,
                        None => panic!("Unnamed field is not supported"),
                    };
                    let f_type = &f.ty;
                    match name.to_string().as_str() {
                        "inner" => quote! {},
                        "played" => quote! {},
                        "byte_per_millisecond" => quote! {},
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
