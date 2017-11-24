use syn;
use std::collections::HashMap;
use proc_macro::TokenStream;


/// Util to parse a derive input from a derive decored token.
pub fn parse_derive_input(input: TokenStream) -> syn::DeriveInput {
    let s = input.to_string();
    syn::parse_derive_input(&s).unwrap()
}


/// Util to parse derive attributes from a derive input.
pub fn parse_derive_attibutes<'a>(ast: syn::DeriveInput) -> HashMap<String, syn::MetaItem> {
    ast.attrs.iter().map(|x| (x.name().to_owned(), x.value.clone())).collect()
}
