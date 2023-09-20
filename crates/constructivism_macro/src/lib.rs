// use constructivist::implement_constructivism_macro;
// implement_constructivism_macro! { "constructivism", 16 }
use proc_macro as pm;
use syn::{parse_macro_input, DeriveInput};
use constructivist::prelude::*;

const DEFAULT_CONSTRUCT_FIELD_LIMIT: u8 = 16;


#[proc_macro_derive(Construct, attributes(construct, required, default))]
pub fn construct_derive(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match DeriveConstruct::from_derive(input) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    let stream = match constructable.build(&Context::new("constructivism")) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
}
#[proc_macro_derive(Mixin, attributes(required, default))]
pub fn segment_derive(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match DeriveSegment::from_derive(input) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    let stream = match constructable.build(&Context::new("constructivism")) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
}

#[proc_macro]
pub fn derive_construct(input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as DeriveConstruct);
    let stream = match input.build(&Context::new("constructivism")) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
}

#[proc_macro]
pub fn construct(input: pm::TokenStream) -> pm::TokenStream {
    let cst = parse_macro_input!(input as Construct);
    let ctx = Context::new("constructivism");
    pm::TokenStream::from(match cst.build(&ctx) {
        Ok(r) => r,
        Err(e) => e.to_compile_error()
    })
}

#[proc_macro]
pub fn implement_constructivism_core(_: pm::TokenStream) -> pm::TokenStream {
    pm::TokenStream::from(genlib::implement_constructivism_core(
        DEFAULT_CONSTRUCT_FIELD_LIMIT,
    ))
}

#[proc_macro]
pub fn implement_constructivism(_: pm::TokenStream) -> pm::TokenStream {
    pm::TokenStream::from(genlib::implement_constructivism(
        DEFAULT_CONSTRUCT_FIELD_LIMIT,
    ))
}
