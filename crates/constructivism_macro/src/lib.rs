#[proc_macro_derive(Construct, attributes(prop, construct, required, default))]
pub fn construct_derive(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::{parse_macro_input, DeriveInput};
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match DeriveConstruct::from_derive(input) {
        Err(e) => return ::proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    let stream = match constructable.build(&Context::new("constructivism")) {
        Err(e) => return ::proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    ::proc_macro::TokenStream::from(stream)
}

#[proc_macro_derive(Segment, attributes(prop, required, default))]
pub fn segment_derive(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::{parse_macro_input, DeriveInput};
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match DeriveSegment::from_derive(input) {
        Err(e) => return ::proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    let stream = match constructable.build(&Context::new("constructivism")) {
        Err(e) => return ::proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    ::proc_macro::TokenStream::from(stream)
}

#[proc_macro]
pub fn derive_construct(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::parse_macro_input;
    let input = parse_macro_input!(input as DeriveConstruct);
    let stream = match input.build(&Context::new("constructivism")) {
        Err(e) => return ::proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    ::proc_macro::TokenStream::from(stream)
}

#[proc_macro]
pub fn derive_segment(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::parse_macro_input;
    let input = parse_macro_input!(input as DeriveSegment);
    let stream = match input.build(&Context::new("constructivism")) {
        Err(e) => return ::proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    ::proc_macro::TokenStream::from(stream)
}

#[proc_macro]
pub fn construct(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::parse_macro_input;
    let cst = parse_macro_input!(input as Construct);
    let ctx = Context::new("constructivism");
    ::proc_macro::TokenStream::from(match cst.build(&ctx) {
        Ok(r) => r,
        Err(e) => e.to_compile_error(),
    })
}
#[proc_macro]
pub fn prop(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::parse_macro_input;
    let cst = parse_macro_input!(input as Prop);
    let ctx = Context::new("constructivism");
    ::proc_macro::TokenStream::from(match cst.build(&ctx) {
        Ok(r) => r,
        Err(e) => e.to_compile_error(),
    })
}

#[proc_macro]
pub fn implement_constructivism_core(
    input: ::proc_macro::TokenStream,
) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::parse_macro_input;
    let limits = parse_macro_input!(input as genlib::ConstructivistLimits);
    ::proc_macro::TokenStream::from(genlib::implement_constructivism_core(limits.max_fields))
}

#[proc_macro]
pub fn implement_constructivism(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::parse_macro_input;
    let limits = parse_macro_input!(input as genlib::ConstructivistLimits);
    ::proc_macro::TokenStream::from(genlib::implement_constructivism(limits.max_fields))
}
