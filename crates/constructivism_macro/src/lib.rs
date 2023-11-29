#[proc_macro_derive(Construct, attributes(prop, param, construct))]
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

#[proc_macro_derive(Segment, attributes(prop, param))]
pub fn segment_derive(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::{parse_macro_input, DeriveInput};
    let input = parse_macro_input!(input as DeriveInput);
    type ConstructivismContext = Context;
    let ctx = ConstructivismContext::new("constructivism");
    let constructable = match DeriveSegment::from_derive(input) {
        Err(e) => return ::proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    let stream = match constructable.build(&ctx) {
        Err(e) => return ::proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    ::proc_macro::TokenStream::from(stream)
}

#[proc_macro]
pub fn derive_construct(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::parse_macro_input;
    type ConstructivismContext = Context;
    let ctx = ConstructivismContext::new("constructivism");
    let input = parse_macro_input!(input as DeriveConstruct);
    let stream = match input.build(&ctx) {
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
    type ConstructivismContext = Context;
    let ctx = ConstructivismContext::new("constructivism");
    let stream = match input.build(&ctx) {
        Err(e) => return ::proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c,
    };
    ::proc_macro::TokenStream::from(stream)
}

#[proc_macro]
pub fn construct(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::parse_macro_input;
    type ConstructivismValue = syn::Expr;
    type ConstructivismContext = Context;
    let cst = parse_macro_input!(input as Construct<ConstructivismValue>);
    ::proc_macro::TokenStream::from(
        match ::constructivist::proc::build(
            ConstructivismContext::new("constructivism"),
            move |ctx| cst.build(ctx),
        ) {
            Ok(r) => r,
            Err(e) => e.to_compile_error(),
        },
    )
    // let mut ctx = ConstructivismContext::new("constructivism");
    // ::proc_macro::TokenStream::from(match cst.build(Ref::new(&mut ctx)) {
    //     Ok(r) => r,
    //     Err(e) => e.to_compile_error(),
    // })
}
#[proc_macro]
pub fn prop(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
    use ::constructivist::prelude::*;
    use ::syn::parse_macro_input;
    let cst = parse_macro_input!(input as Prop);
    type ConstructivismContext = Context;
    let ctx = ConstructivismContext::new("constructivism");
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
