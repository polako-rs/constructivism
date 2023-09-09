use proc_macro as pm;
use syn::ItemImpl;
use syn::{parse_macro_input, DeriveInput};

mod buildlib;
mod derive;
mod synext;
use buildlib::implement_construct_core;
use derive::lib;
use derive::ConstructMode;
use derive::{Constructable, Methods};

#[proc_macro_derive(Construct, attributes(extends, mixin, required, default))]
pub fn derive_construct_item(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match Constructable::from_derive(input, ConstructMode::object()) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    let stream = match constructable.build(lib()) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
}
#[proc_macro_derive(Mixin, attributes(required, default))]
pub fn derive_mixin(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match Constructable::from_derive(input, ConstructMode::mixin()) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    let stream = match constructable.build(lib()) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
}

#[proc_macro]
pub fn constructable(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as Constructable);
    let stream = match input.build(lib()) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
}
#[proc_macro_attribute]
pub fn construct_methods(_: pm::TokenStream, input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    let methods = match Methods::from_input(input) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(r) => r,
    };
    let stream = match methods.build(lib()) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
}

#[proc_macro]
pub fn construct_implementations(_: pm::TokenStream) -> pm::TokenStream {
    pm::TokenStream::from(implement_construct_core())
}
