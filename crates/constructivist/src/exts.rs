use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Type};

pub trait TypeExt {
    fn as_ident(&self) -> syn::Result<Ident>;
    fn is_nothing(&self) -> bool;
}
impl TypeExt for Type {
    fn as_ident(&self) -> syn::Result<Ident> {
        let Type::Path(path) = &self else {
            return Err(syn::Error::new(self.span(), format!(
                "Can't extract ident from type {}",
                quote!({#self}).to_string()
            )))
        };
        if path.path.segments.is_empty() {
            return Err(syn::Error::new(
                self.span(),
                format!(
                    "Can't extract ident from type {}",
                    quote!({#self}).to_string()
                ),
            ));
        }
        Ok(path.path.segments.last().unwrap().ident.clone())
    }
    fn is_nothing(&self) -> bool {
        &self.into_token_stream().to_string() == "Nothing"
    }
}
