use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{Data, Generics};

use crate::{
    IGNORED_FIELDS,
    custom_input_types::{CustomAdditionalInput, CustomInputTypesMap},
};

pub fn struct_lazy_playlist_child(
    inner_name: &Ident,
    name: &Ident,
    generics: &Generics,
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
    let recursive2 = custom_types.additional_inputs.iter().map(
        |CustomAdditionalInput {
             name,
             input_type,
             optional,
             ..
         }| {
            if *optional {
                quote_spanned! {name.span()=>
                    #name: Option<#input_type>,
                }
            } else {
                quote! {
                    #name: #input_type,
                }
            }
        },
    );
    let recursive = quote! {
        #recursive
        #(#recursive2)*
    };

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        // The generated struct.
        pub struct #name #generics #where_clause {
            inner: Option< #inner_name #ty_generics>,
            #recursive
        }
    }
}
