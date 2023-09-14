use proc_macro as pm;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemImpl;
use syn::{parse_macro_input, DeriveInput};

const DEFAULT_CONSTRUCT_FIELD_LIMIT: u8 = 16;

use constructivist::derive::{ConstructMode, Constructable, Protocols};
use constructivist::genlib;
use toml::Value;

fn lib() -> TokenStream {
    let lib = quote! { ::constructivism::core };
    let Some(manifest_path) = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map(|mut path| { path.push("Cargo.toml"); path })
        else { return lib };
    let Ok(manifest) = std::fs::read_to_string(&manifest_path) else {
        return lib
    };
    let Ok(manifest) = toml::from_str::<toml::map::Map<String, toml::Value>>(&manifest) else {
        return lib
    };

    let Some(pkg) = manifest.get("package") else { return lib };
    let Some(pkg) = pkg.as_table() else { return lib };
    if Some(&Value::String("constructivism_macro".to_string())) == pkg.get("name") {
        return quote! { ::constructivism::core };
    }
    let Some(pkg) = pkg.get("name") else { return lib };
    let Some(pkg) = pkg.as_str() else { return lib };
    if pkg.trim() == "constructivism_macro" {
        quote! { ::constructivism::core }
    } else {
        lib
    }
}

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
pub fn construct_protocols(_: pm::TokenStream, input: pm::TokenStream) -> pm::TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    let protocols = match Protocols::from_input(input) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(r) => r,
    };
    let stream = match protocols.build(lib()) {
        Err(e) => return pm::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    pm::TokenStream::from(stream)
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
