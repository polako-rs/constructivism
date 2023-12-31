use std::str::FromStr;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse::Parse, parse_macro_input, LitStr, Token, Type};

struct ConstructivismSettnigs {
    pub domain: String,
    pub value_type: String,
    pub context_type: String,
}

impl Parse for ConstructivismSettnigs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let domain: LitStr = input.parse()?;
        let domain = domain.value();
        let mut value_type = format!("::syn::Expr");
        let mut context_type = format!("Context");
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let vt = input.parse::<Type>()?;
            let vt = quote! { #vt };
            value_type = vt.to_string();
        }
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let vt = input.parse::<Type>()?;
            let vt = quote! { #vt };
            context_type = vt.to_string();
        }
        Ok(ConstructivismSettnigs {
            domain,
            value_type,
            context_type,
        })
    }
}

#[proc_macro]
pub fn implement_constructivism_macro(input: TokenStream) -> TokenStream {
    let settings = parse_macro_input!(input as ConstructivismSettnigs);
    // let path = settings.domain;
    // let path = format!("\"{path}\"");
    let source = include_str!("constructivism_macro.include");
    let exact_domain = format!("\"{}\"", settings.domain);
    let source = source.replace("\"constructivism\"", &exact_domain);
    let exact_value = format!("type ConstructivismValue = {};", settings.value_type);
    let source = source.replace("type ConstructivismValue = syn::Expr;", &exact_value);
    let exact_context = format!("type ConstructivismContext = {};", settings.context_type);
    let source = source.replace("type ConstructivismContext = Context;", &exact_context);
    match TokenStream::from_str(&source) {
        Err(e) => {
            let e = e.to_string();
            TokenStream::from(quote! { compile_error!("Can't parse constructivism_macro: {}", #e) })
        }
        Ok(stream) => stream,
    }
}
