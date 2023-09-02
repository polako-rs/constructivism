use proc_macro2::Ident;
use syn::Type;

pub trait TypeExt {
    fn as_ident(&self) -> Option<&Ident>;
}
impl TypeExt for Type {
    fn as_ident(&self) -> Option<&Ident> {
        let Type::Path(path) = &self else {
            return None;
        };
        if path.path.segments.is_empty() {
            return None;
        }
        Some(&path.path.segments.last().unwrap().ident)
    }
}