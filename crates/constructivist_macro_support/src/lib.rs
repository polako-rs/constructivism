use proc_macro;
use syn::{ parse_macro_input, DeriveInput };

mod synext;
mod derive;
mod buildlib;
use derive::Constructable;
use derive::ConstructMode;
use derive::lib;
use buildlib::implement_construct_core;



#[proc_macro_derive(Construct, attributes(extends, mixin, required, default))]
pub fn derive_construct_item(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match Constructable::from_derive(input, ConstructMode::object()) {
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    proc_macro::TokenStream::from(constructable.build(lib()))
}
#[proc_macro_derive(Mixin, attributes(required, default))]
pub fn derive_mixin(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match Constructable::from_derive(input, ConstructMode::mixin()) {
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    proc_macro::TokenStream::from(constructable.build(lib()))
}

#[proc_macro]
pub fn constructable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Constructable);
    proc_macro::TokenStream::from(input.build(lib()))
}

#[proc_macro]
pub fn construct_implementations(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(implement_construct_core())
}
