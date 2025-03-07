use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{Data, Generics};

use crate::{
    IGNORED_FIELDS,
    custom_input_types::{CustomAdditionalInput, CustomInputTypesMap},
};

pub fn new_lazy_playlist_child(
    name: &Ident,
    generics: &Generics,
    data: &Data,
    custom_types: &CustomInputTypesMap,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let input = new_lazy_playlist_child_input(data, custom_types);
    let constructor = new_lazy_playlist_child_constructor(data, custom_types);

    quote! {
        // The generated impl.
        impl #impl_generics #name #ty_generics #where_clause {
            pub async fn new(#input) -> anyhow::Result<Self> {
                Ok(#constructor)
            }
        }
    }
}

fn new_lazy_playlist_child_constructor(
    data: &Data,
    custom_types: &CustomInputTypesMap,
) -> proc_macro2::TokenStream {
    let recursive = match data {
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
                    inner: None,
                    #(#recurse)*
                }
            }
            _ => panic!("Only named struct is supported"),
        },
        _ => panic!("Only struct is supported"),
    };

    let recursive2 = custom_types.additional_inputs.iter().map(
        |CustomAdditionalInput {
             name,
             default,
             optional,
             ..
         }| {
            if *optional {
                quote_spanned! {name.span()=>
                    #name: Some(#name),
                }
            } else {
                quote! {
                    #name: #name.unwrap_or(#default),
                }
            }
        },
    );
    quote! {
        Self {
            #recursive
            #(#recursive2)*
        }
    }
}

fn new_lazy_playlist_child_input(
    data: &Data,
    custom_types: &CustomInputTypesMap,
) -> proc_macro2::TokenStream {
    let recursive = match data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = match &f.ident {
                        Some(name) => name,
                        None => panic!("Unnamed field is not supported"),
                    };
                    let f_type = match custom_types.input_types.get(&name.to_string()) {
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
    };
    let recursive2 = custom_types.additional_inputs.iter().map(
        |CustomAdditionalInput {
             name,
             input_type,
             optional,
             ..
         }| {
            if *optional {
                quote_spanned! {name.span()=>
                    #name: #input_type,
                }
            } else {
                quote! {
                    #name: Option<#input_type>,
                }
            }
        },
    );
    quote! {
        #recursive
        #(#recursive2)*
    }
}
