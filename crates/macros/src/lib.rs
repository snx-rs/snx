mod validation;

use proc_macro::TokenStream;
use syn::DeriveInput;

use crate::validation::expand_validate;

/// Generates code which validates the given struct
#[proc_macro_derive(Validate, attributes(rule))]
pub fn validate(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match expand_validate(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

// #[proc_macro_attribute]
// pub fn error(_: TokenStream, item: TokenStream) -> TokenStream {
//     item
// }
