use std::collections::HashMap;

use darling::{FromDeriveInput, FromMeta};

#[derive(FromDeriveInput)]
#[darling(attributes(custom_input_type), supports(struct_named))]
pub struct CustomInputTypes {
    #[darling(default, multiple, rename = "input_type")]
    pub input_types: Vec<CustomInputType>,
    #[darling(default, multiple, rename = "additional_input")]
    pub additional_inputs: Vec<CustomAdditionalInput>,
}
#[derive(FromMeta, Clone)]
pub struct CustomInputType {
    pub name: String,
    pub input_type: syn::Type,
}

#[derive(FromMeta, Clone)]
pub struct CustomAdditionalInput {
    pub name: syn::Ident,
    pub input_type: syn::Type,
    pub default: syn::Expr,
    #[darling(default)]
    pub optional: bool,
}

pub struct CustomInputTypesMap {
    pub input_types: HashMap<String, syn::Type>,
    pub additional_inputs: Vec<CustomAdditionalInput>,
}

impl Into<CustomInputTypesMap> for CustomInputTypes {
    fn into(self) -> CustomInputTypesMap {
        let input_types = self
            .input_types
            .into_iter()
            .map(|input_type| (input_type.name, input_type.input_type))
            .collect();
        CustomInputTypesMap {
            input_types,
            additional_inputs: self.additional_inputs,
        }
    }
}
