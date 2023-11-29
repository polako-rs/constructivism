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
            return Err(syn::Error::new(
                self.span(),
                format!(
                    "Can't extract ident from type {}",
                    quote!({#self}).to_string()
                ),
            ));
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

pub trait Capitalize {
    type Output;
    fn capitalize(&self) -> Self::Output;
}

impl<T: AsRef<str>> Capitalize for T {
    type Output = String;
    fn capitalize(&self) -> Self::Output {
        self.as_ref()
            .chars()
            .enumerate()
            .map(
                |(idx, ch)| {
                    if idx > 0 {
                        ch
                    } else {
                        ch.to_ascii_uppercase()
                    }
                },
            )
            .collect()
    }
}

pub trait ToCamelCase {
    type Output;
    fn to_camel_case(&self) -> Self::Output;
}

impl<'a> ToCamelCase for &'a str {
    type Output = String;
    fn to_camel_case(&self) -> Self::Output {
        self.split("_").map(|s| s.capitalize()).collect()
    }
}

impl ToCamelCase for String {
    type Output = String;
    fn to_camel_case(&self) -> Self::Output {
        self.split("_").map(|s| s.capitalize()).collect()
    }
}

impl ToCamelCase for Ident {
    type Output = Ident;
    fn to_camel_case(&self) -> Self::Output {
        let s = self.to_string().to_camel_case();
        Ident::new(&s, self.span())
    }
}

pub trait Suffix {
    type Output;
    fn suffix<S: AsRef<str>>(&self, suffix: S) -> Self::Output;
}

impl<'a> Suffix for &'a str {
    type Output = String;
    fn suffix<S: AsRef<str>>(&self, suffix: S) -> Self::Output {
        format!("{}{}", self, suffix.as_ref())
    }
}
impl Suffix for String {
    type Output = String;
    fn suffix<S: AsRef<str>>(&self, suffix: S) -> Self::Output {
        format!("{}{}", self, suffix.as_ref())
    }
}
impl Suffix for Ident {
    type Output = Ident;
    fn suffix<S: AsRef<str>>(&self, suffix: S) -> Self::Output {
        Ident::new(&self.to_string().suffix(suffix), self.span())
    }
}
