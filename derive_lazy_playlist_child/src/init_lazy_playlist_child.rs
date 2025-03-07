use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{Data, Generics};

use crate::{
    IGNORED_FIELDS,
    custom_input_types::{CustomAdditionalInput, CustomInputTypesMap},
};

pub fn init_lazy_playlist_child(
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
                    if IGNORED_FIELDS.contains(&name.to_string().as_str()) {
                        return quote! {};
                    }
                    match name.to_string().as_str() {
                        "shuffle" => quote_spanned! {name.span()=>
                            self.#name,
                        },
                        "repeat" => quote_spanned! {name.span()=>
                            self.#name,
                        },
                        _ => {
                            let err = format!("{} is none", name);
                            quote_spanned! {name.span()=>
                                match self.#name.take() {
                                    Some(d) => d,
                                    None => return Err(anyhow::anyhow!(#err)),
                                },
                            }
                        }
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
        |CustomAdditionalInput { name, optional, .. }| {
            if *optional {
                let err = format!("{} is none", name);
                quote_spanned! {name.span()=>
                    match self.#name.take() {
                        Some(d) => d,
                        None => return Err(anyhow::anyhow!(#err)),
                    },
                }
            } else {
                quote! {
                    self.#name,
                }
            }
        },
    );
    let recursive = quote! {
        #recursive
        #(#recursive2)*
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        // The generated impl.
        impl #impl_generics #name #ty_generics #where_clause {
            async fn init(&mut self) -> anyhow::Result<()> {
                if self.inner.is_none() {
                    self.inner = Some(#inner_name::new(#recursive).await?);
                }
                Ok(())
            }
        }
    }
}
