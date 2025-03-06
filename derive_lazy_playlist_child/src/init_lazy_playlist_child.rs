use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{Data, Generics};

pub fn init_lazy_playlist_child(
    inner_name: &Ident,
    name: &Ident,
    generics: &Generics,
    data: &Data,
) -> proc_macro2::TokenStream {
    let recursive = match data {
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
                            self.#name,
                        },
                        "repeat" => quote_spanned! {name.span()=>
                            self.#name,
                        },
                        _ => quote_spanned! {name.span()=>
                            match self.#name.take() {
                                Some(d) => d,
                                // TODO find a way to turn this into static error
                                None => return Err(anyhow::anyhow!(format!("{} is none", stringify!(#name)))),
                            },
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
