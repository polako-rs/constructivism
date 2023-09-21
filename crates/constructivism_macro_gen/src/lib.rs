use std::str::FromStr;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, LitStr};

#[proc_macro]
pub fn implement_constructivism_macro(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as LitStr).value();
    let path = format!("\"{path}\"");
    let source = include_str!("../../constructivism_macro/src/lib.rs");
    let source = source.replace("\"constructivism\"", &path);
    match TokenStream::from_str(&source) {
        Err(e) => { 
            let e = e.to_string();
            TokenStream::from(quote! { compile_error!("Can't parse constructivism_macro: {}", #e) })
        },
        Ok(stream) => stream
    }
}