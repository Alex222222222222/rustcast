use std::collections::HashMap;

use custom_input_types::CustomInputTypes;
use darling::FromDeriveInput;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

mod custom_input_types;
mod impl_playlist_child;
mod init_lazy_playlist_child;
mod new_lazy_playlist_child;
mod struct_lazy_playlist_child;

const IGNORED_FIELDS: [&str; 8] = [
    "inner",
    "played",
    "current_index",
    "byte_per_millisecond",
    "title",
    "artist",
    "content_type",
    "current_stream",
];

#[proc_macro_derive(LazyPlaylistChild, attributes(custom_input_type))]
pub fn derive_lazy_playlist_child(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let input_types: HashMap<String, syn::Type> =
        CustomInputTypes::from_derive_input(&input).unwrap().into();

    // Used in the quasi-quotation below as `#name`.
    let inner_name = input.ident;
    if !inner_name.to_string().ends_with("Inner") {
        panic!("Must be a inner struct, to derive LazyPlaylistChild");
    }
    let name = syn::Ident::new(
        &inner_name.to_string()[..inner_name.to_string().len() - 5],
        inner_name.span(),
    );

    // Add a bound `T: HeapSize` to every type parameter T.
    let generics = input.generics;

    let struct_lazy_playlist_child = struct_lazy_playlist_child::struct_lazy_playlist_child(
        &inner_name,
        &name,
        &generics,
        &input.data,
        &input_types,
    );
    let new_lazy_playlist_child = new_lazy_playlist_child::new_lazy_playlist_child(
        &name,
        &generics,
        &input.data,
        &input_types,
    );
    let init_lazy_playlist_child = init_lazy_playlist_child::init_lazy_playlist_child(
        &inner_name,
        &name,
        &generics,
        &input.data,
    );
    let impl_playlist_child = impl_playlist_child::impl_playlist_child(&name, &generics);

    let expanded = quote! {
        #struct_lazy_playlist_child

        #new_lazy_playlist_child

        #init_lazy_playlist_child

        #impl_playlist_child
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
