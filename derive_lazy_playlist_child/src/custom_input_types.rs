use std::collections::HashMap;

use darling::{FromDeriveInput, FromMeta};

#[derive(FromDeriveInput)]
#[darling(attributes(custom_input_type), supports(struct_named))]
pub struct CustomInputTypes {
    #[darling(default, multiple, rename = "input_type")]
    pub input_types: Vec<CustomInputType>,
}
#[derive(FromMeta, Clone)]
pub struct CustomInputType {
    pub name: String,
    pub input_type: syn::Type,
}

impl Into<HashMap<String, syn::Type>> for CustomInputTypes {
    fn into(self) -> HashMap<String, syn::Type> {
        self.input_types
            .into_iter()
            .map(|input_type| (input_type.name, input_type.input_type))
            .collect()
    }
}
