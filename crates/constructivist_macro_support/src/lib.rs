use syn::{parse_macro_input, DeriveInput, Data, Type, Expr, parse::{Parse, Parser}, bracketed, punctuated::Punctuated, Token, Ident, parenthesized};
use quote::*;
use proc_macro;
use proc_macro2::TokenStream;

mod synext;
use synext::*;

fn lib() -> TokenStream {
    let lib = quote! { ::constructivist_core };
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
    let Some(pkg) = pkg.get("name") else { return lib };
    let Some(pkg) = pkg.as_str() else { return lib };
    if pkg.trim() == "constructivist" {
        quote! { ::constructivist_core }
    } else {
        lib
    }
}


#[proc_macro_derive(Construct, attributes(wraps, required, default))]
pub fn derive_construct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    proc_macro::TokenStream::from(derive_construct_impl(input, lib()))
}

fn derive_construct_impl(input: DeriveInput, lib: TokenStream) -> TokenStream {
    if input.generics.params.len() > 0 {
        return quote!(compile_error!("#[derive(Construct)] doesn't support generics yet."))
    }
    let type_ident = &input.ident;                      // Slider
    let mod_ident = format_ident!(                      // slider_construct
        "{}_construct",
        type_ident.to_string().to_lowercase()
    );
    let wraps: Type = if let Some(wraps) = input.attrs.iter().find(|a| a.path().is_ident("wraps")) {
        wraps.parse_args().expect("Expected type path.")
    } else {
        syn::parse2(quote!(())).unwrap()
    };

    let Data::Struct(input) = input.data else {
        return quote!(compile_error!("#[derive(Construct)] only supports named structs. You can use `constructable!` for complex cases."))
    };
    let mut type_props = quote! { };                   // slider_construct::min, slider_construct::max, slider_construct::val,
    let mut type_props_deconstruct = quote! { };       // slider_construct::min(min), slider_construct::max(max), slider_construct::val(val),
    let mut param_values = quote! { };                  // min, max, val,
    let mut impls = quote! { };
    let mut fields = quote! { };
    let mut fields_new = quote! { };

    for field in input.fields.iter() {
        let Some(ident) = &field.ident else {
            return quote!(compile_error!("#[derive(Construct)] only supports named structs. You can use `constructable!` for complex cases."))
        };
        let field_ty = &field.ty;
        type_props = quote! { #type_props #mod_ident::#ident, };
        type_props_deconstruct = quote! { #type_props_deconstruct #mod_ident::#ident(#ident), };
        param_values = quote! { #param_values #ident, };
        fields = quote! { #fields
            #[allow(unused_variables)]
            pub #ident: #lib::Prop<#ident, #field_ty>,
        };
        fields_new = quote! { #fields_new #ident: #lib::Prop(::std::marker::PhantomData), };

        let default = if field.attrs.iter().any(|a| a.path().is_ident("required")) {
            quote! { }
        } else if let Some(default) = field.attrs.iter().find(|a| a.path().is_ident("default")) {
            let Ok(expr) = default.parse_args::<Expr>() else {
                return quote!(compile_error!("Invalid expression for #[default(expr)]."))
            };
            quote! {
                impl Default for #ident {
                    fn default() -> Self {
                        #ident(#expr)
                    }
                }
            }
        } else {
            quote! { #[derive(Default)] }
        };
        impls = quote! { #impls 
            #default
            #[allow(non_camel_case_types)]
            pub struct #ident(pub #field_ty);
            impl<T: Into<#field_ty>> From<T> for #ident {
                fn from(__value__: T) -> Self {
                    #ident(__value__.into())
                }
            }
            impl #lib::AsField for #ident {
                fn as_field() -> #lib::Field<Self> {
                    #lib::Field::new()
                }
            }
        }
    }
    quote! {
        mod #mod_ident {
            use super::*;
            pub struct Fields {
                #fields
            }
            impl #lib::Singleton for Fields {
                fn instance() -> &'static Self {
                    &Fields {
                        #fields_new
                    }
                }
            }
            impl std::ops::Deref for Fields {
                type Target = <<super::#type_ident as #lib::Construct>::Wraps as #lib::Construct>::Fields;
                fn deref(&self) -> &Self::Target {
                    <<<super::#type_ident as #lib::Construct>::Wraps as #lib::Construct>::Fields as #lib::Singleton>::instance()
                }
            }
            #impls
        }
        impl #lib::NonUnit for #type_ident { }
        impl #lib::Construct for #type_ident {
            type Fields = #mod_ident::Fields;
            type Props = ( #type_props );
            type Wraps = #wraps;
            type Wrapped = (Self, <Self::Wraps as #lib::Construct>::Wrapped);
            type WrappedProps = (#type_props <Self::Wraps as #lib::Construct>::WrappedProps);
            fn construct_fields() -> &'static Self::Fields {
                <#mod_ident::Fields as #lib::Singleton>::instance()
            }
            fn construct(props: Self::Props) -> Self {
                let (#type_props_deconstruct) = props;
                Self { #param_values }
            }
            fn construct_all<P>(props: P) -> <Self as #lib::Construct>::Wrapped
            where Self: Sized, P: #lib::DefinedValues<
                Self::Props,
                Output = <<<Self as #lib::Construct>::Wraps as #lib::Construct>::WrappedProps as #lib::AsProps>::Defined 
            > {
                let ((args), props) = props.extract_values();
                (Self::construct(args), <<Self as #lib::Construct>::Wraps as #lib::Construct>::construct_all(props))
            }
        }
    }
}

enum PropType {
    Single(Type),
    Union(Vec<Prop>),
}
impl Parse for PropType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let content;
            bracketed!(content in input);
            let props = content.parse_terminated(Prop::parse, Token![,])?;
            Ok(PropType::Union(props.into_iter().collect()))
        } else {
            Ok(PropType::Single(input.parse()?))
        }
    }
}
struct Prop {
    name: Ident,
    ty: PropType,
    default: Option<Expr>,
}

impl Parse for Prop {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let mut default = None;
        if input.peek(Token![=]) {
            input.parse::<Token![=]>();
            default = Some(input.parse()?);
        }
        Ok(Prop { name, ty, default })
    }
}

struct Constructable {
    ty: Type,
    extends: Option<Type>,
    props: Vec<Prop>,
    body: Expr
}

impl Parse for Constructable {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;
        let mut extends = None;
        if let Ok(ident) = input.parse::<Ident>() {
            if &ident.to_string() != "extends" {
                return Err(syn::Error::new(ident.span(), "Expected `extends` ident"));
            }
            extends = Some(input.parse()?)
        }
        let content;
        parenthesized!(content in input);
        let props = content.parse_terminated(Prop::parse, Token![,])?;
        let props = props.into_iter().collect();
        let body = input.parse()?;
        Ok(Constructable { ty, extends, props, body })
    }
}

impl Constructable {
    fn build(&self, lib: TokenStream) -> TokenStream {
        let ty = &self.ty;
        let Some(type_ident) = ty.as_ident() else {
            return quote!(compile_error!("Can't implement Construct for {}", stringify!(#ty)));
        };
        let mod_ident = format_ident!(                      // slider_construct
            "{}_construct",
            type_ident.to_string().to_lowercase()
        );
        let extends = self.extends.clone().unwrap_or(syn::parse2(quote!(())).unwrap());
        let mut type_props = quote! { };                   // slider_construct::min, slider_construct::max, slider_construct::val,
        let mut type_props_deconstruct = quote! { };       // slider_construct::min(min), slider_construct::max(max), slider_construct::val(val),
        let mut param_values = quote! { };                  // min, max, val,
        let mut impls = quote! { };
        let mut fields = quote! { };
        let mut fields_new = quote! { };
        let construct = &self.body;

        for prop in self.props.iter() {
            let PropType::Single(prop_ty) = &prop.ty else {
                return quote!(compile_error!("Union props not supported yet."))
            };
            let ident = &prop.name;
            type_props = quote! { #type_props #mod_ident::#ident, };
            type_props_deconstruct = quote! { #type_props_deconstruct #mod_ident::#ident(mut #ident), };
            fields = quote! { #fields
                #[allow(unused_variables)]
                pub #ident: #lib::Prop<#ident, #prop_ty>,
            };
            fields_new = quote! { #fields_new #ident: #lib::Prop(::std::marker::PhantomData), };
            let default = if let Some(default) = &prop.default {
                quote! { 
                    impl Default for #ident {
                        fn default() -> Self {
                            #ident(#default)
                        }
                    }
                }
            } else {
                quote! { }
            };
            impls = quote! { #impls 
                #default
                #[allow(non_camel_case_types)]
                pub struct #ident(pub #prop_ty);
                impl<T: Into<#prop_ty>> From<T> for #ident {
                    fn from(__value__: T) -> Self {
                        #ident(__value__.into())
                    }
                }
                impl #lib::AsField for #ident {
                    fn as_field() -> #lib::Field<Self> {
                        #lib::Field::new()
                    }
                }
            };
        }
        quote! {
            mod #mod_ident {
                use super::*;
                pub struct Fields {
                    #fields
                }
                impl #lib::Singleton for Fields {
                    fn instance() -> &'static Self {
                        &Fields {
                            #fields_new
                        }
                    }
                }
                impl std::ops::Deref for Fields {
                    type Target = <<super::#type_ident as #lib::Construct>::Wraps as #lib::Construct>::Fields;
                    fn deref(&self) -> &Self::Target {
                        <<<super::#type_ident as #lib::Construct>::Wraps as #lib::Construct>::Fields as #lib::Singleton>::instance()
                    }
                }
                #impls
            }
            impl #lib::NonUnit for #type_ident { }
            impl #lib::Construct for #type_ident {
                type Fields = #mod_ident::Fields;
                type Props = ( #type_props );
                type Wraps = #extends;
                type Wrapped = (Self, <Self::Wraps as #lib::Construct>::Wrapped);
                type WrappedProps = (#type_props <Self::Wraps as #lib::Construct>::WrappedProps);
                fn construct_fields() -> &'static Self::Fields {
                    <#mod_ident::Fields as #lib::Singleton>::instance()
                }
                fn construct(props: Self::Props) -> Self {
                    let (#type_props_deconstruct) = props;
                    #construct
                }
                fn construct_all<P>(props: P) -> <Self as #lib::Construct>::Wrapped
                where Self: Sized, P: #lib::DefinedValues<
                    Self::Props,
                    Output = <<<Self as #lib::Construct>::Wraps as #lib::Construct>::WrappedProps as #lib::AsProps>::Defined 
                > {
                    let ((args), props) = props.extract_values();
                    (Self::construct(args), <<Self as #lib::Construct>::Wraps as #lib::Construct>::construct_all(props))
                }
            }
        }
    }
}


#[proc_macro]
pub fn constructable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Constructable);
    proc_macro::TokenStream::from(input.build(lib()))
}

#[proc_macro]
pub fn construct_implementations(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let max_size = 8;
    let extract_field_impls = impl_all_extract_field(max_size);
    let add_to_props = impl_all_add_to_props(max_size);
    let defined_values = impl_all_defined_values(max_size);
    let join_props = impl_all_join_props(max_size);
    let as_flat_props = impl_all_as_flat_props(max_size);
    let defined = impl_all_defined(max_size);
    proc_macro::TokenStream::from(quote! {
        #extract_field_impls
        #add_to_props
        #defined_values
        #join_props
        #as_flat_props
        #defined
    })
}


fn impl_all_extract_field(max_size: u8) -> TokenStream {
    let mut out = quote! { };
    for size in 1..max_size + 1 {
        for idx in 0..size {
            let impl_extract = impl_extract_field(idx, size);
            out = quote! { #out #impl_extract }
        }
    }
    out
}
fn impl_all_add_to_props(max_size: u8) -> TokenStream {
    let mut out = quote! { };
    for size in 1..max_size + 1 {
        for idx in 0..size {
            let impl_add_to_props = impl_add_to_props(idx, size);
            out = quote! { #out #impl_add_to_props }
        }
    }
    out
}
fn impl_all_defined_values(max_size: u8) -> TokenStream {
    let mut out = quote! { };
    for s in 0..max_size {
        for d in 0..s+1 {
            let def = impl_defined_values(d+1, s+1);
            out = quote! { #out #def }
        }
    }
    out
}
fn impl_all_as_flat_props(max_size: u8) -> TokenStream {
    let mut out = quote! { };
    for size in 1..max_size+1 {
        let as_flat_props = impl_as_flat_props(size);
        out = quote! { #out #as_flat_props };
    }
    out
}

fn impl_all_join_props(max_size: u8) -> TokenStream {
    let mut out = quote! { };
    for size in 1..max_size + 1 {
        for shift in 1..size {
            let join_props = impl_join_props(shift, size);
            out = quote! { #out #join_props };
        }
    }
    out
}
fn impl_all_defined(max_size: u8) -> TokenStream {
    let mut out = quote! { };
    for size in 1..max_size + 1 {
        let defined = impl_defined(size);
        out = quote! { #out #defined }
    }
    out
}

/// Generates single ExtractField trait implementation.
/// `impl_extract_field(1, 3) will generate this:
/// ```ignore
/// impl<T1, A0, A1: A<1, T1>, A2> ExtractField<F<1, T1>, T1> for Props<(A0, A1, A2)> {
///     fn field(&self, _: &Field<T1>) -> F<1, T1> {
///         F::<1, T1>(PhantomData)
///     }
/// }
/// ```
fn impl_extract_field(idx: u8, size: u8) -> TokenStream {
    let ti = format_ident!("T{idx}");
    let fi = quote! { F<#idx, #ti> };
    let mut gin = quote! { };
    let mut gout = quote! { };
    for i in 0..size {
        let ai = format_ident!("A{i}");
        if i == idx {
            gin = quote! { #gin #ai: A<#i, #ti>, }
        } else {
            gin = quote! { #gin #ai,}
        }
        gout = quote! { #gout #ai, };
    }

    quote! { 
        impl<#ti, #gin> ExtractField<#fi, #ti> for Props<(#gout)> {
            fn field(&self, _: &Field<#ti>) -> #fi {
                F::<#idx, #ti>(PhantomData)
            }
        }
    }
}

/// Generates single std::ops::Add implementation for Props of size `size`
/// and prop at `idx` position. `impl_add_to_props(1, 4)` will generate this:
/// ```ignore
/// impl<T1, A0, A2, A3> std::ops::Add<D<1, T1>> for Props<(A0, U<1, T1>, A2, A3)> {
///     type Output = Props<(A0, D<1, T1>, A2, A3)>;
///     fn add(self, rhs: D<1, T1>) -> Self::Output {
///         let (p0, _, p2, p3) = self.0;
///         Props((p0, rhs, p2, p3))
///     }
/// }
/// ```
fn impl_add_to_props(idx: u8, size: u8) -> TokenStream {
    let ti = format_ident!("T{idx}");
    let di = quote! { D<#idx, #ti> };
    let ui = quote! { U<#idx, #ti> };
    let mut gin = quote! { };
    let mut pin = quote! { };
    let mut pout = quote! { };
    let mut dcs = quote! { };
    let mut vls = quote! { };
    for i in 0..size {
        if i == idx {
            pin = quote! { #pin #ui, };
            pout = quote! { #pout #di, };
            dcs = quote! { #dcs _, };
            vls = quote! { #vls rhs, };
        } else {
            let ai = format_ident!("A{i}");
            let pi = format_ident!("p{i}");
            gin = quote! { #gin #ai, };
            pin = quote! { #pin #ai, };
            pout = quote! { #pout #ai, };
            dcs = quote! { #dcs #pi, };
            vls = quote! { #vls #pi, };
        }
    }
    quote! {
        impl<#ti, #gin> std::ops::Add<#di> for Props<(#pin)> {
            type Output = Props<(#pout)>;
            fn add(self, rhs: #di) -> Self::Output {
                let (#dcs) = self.0;
                Props((#vls))
            }
        }
    }
}

/// Implement single DefinedValues. `impl_defined_values(2, 4)` will generate
/// ```ignore
/// impl<P0, P1, T0, T1, T2, T3> DefinedValues<(P0, P1, ())> for Props<(T0, T1, T2, T3)>
/// where
///     P0: AsField,
///     P1: AsField,
///     T0: DefinedValue<Value = P0>,
///     T1: DefinedValue<Value = P1>,
///     T2: MoveTo<0>,
///     T3: MoveTo<1>,
/// {
///     type Output = Props<(T2::Target, T3::Target)>;
///     fn extract_values(self) -> ((P0, P1, ()), Self::Output) {
///         let (p0, p1, p2, p3) = self.0;
///         ((
///             p0.extract_value(),
///             p1.extract_value(),
///             (),
///         ), Props((
///             p2.move_to(),
///             p3.move_to(),
///         )))
///     }
/// }
/// ```
fn impl_defined_values(defined: u8, size: u8) -> TokenStream {
    let mut gin = quote! { };
    let mut cnstr = quote! { };
    let mut pfor = quote! { };
    let mut pres = quote! { };
    let mut dcst = quote! { };
    let mut pout = quote! { };
    let mut ex = quote! { };
    let mut mv = quote! { };
    for i in 0..size {
        let pi = format_ident!("P{i}");
        let ti = format_ident!("T{i}");
        let vi = format_ident!("p{i}");
        if i < defined {
            gin = quote! { #gin #pi, };
            cnstr = quote! { #cnstr #ti: DefinedValue<Value = #pi>, };
            cnstr = quote! { #cnstr #pi: AsField, };
            pres = quote! { #pres #pi, };
            ex = quote! { #ex #vi.extract_value(), };
        } else {
            let m = i - defined;
            cnstr = quote! { #cnstr #ti: MoveTo<#m>, };
            pout = quote! { #pout #ti::Target, };
            mv = quote! { #mv #vi.move_to(), };
        }
        dcst = quote!{ #dcst #vi, };
        gin = quote! { #gin #ti, };
        pfor = quote! { #pfor #ti, };
    }
    let debug = format_ident!("debug_defined_values_defined_{defined}_size_{size}");
    let debug_pres = format!("{pres:?}");
    quote! { 
        fn #debug() {
            let debug_pres = #debug_pres;
        }
        impl<#gin> DefinedValues<(#pres)> for Props<(#pfor)> where #cnstr {
            type Output = Props<(#pout)>;
            fn extract_values(self) -> ((#pres), Self::Output) {
                let (#dcst) = self.0;
                ((#ex),Props((#mv)))
            }
        }
    }
}

// impl<T0: AsField, T1: AsField> AsFlatProps for (T0,T1,()) {
//     type Defined = (D<0, T0>, D<1, T1>);
//     type Undefined = (U<0, T0>, U<1, T1>);
//     fn as_flat_props() -> Self::Undefined {
//         (U::<0, _>(PhantomData),U::<1, _>(PhantomData))
//     }
// }
// impl<T0: AsField, T1: AsField, V: JoinProps<(U<0, T0>,U<1, T1>)>, P: AsFlatProps<Undefined = V>> AsFlatProps for (T0, T1, P)
// {
//     type Defined = V::DefinedResult;
//     type Undefined = V::UndefinedResult;
//     fn as_flat_props() -> Self::Undefined {
//         V::join()
//     }
// }
fn impl_as_flat_props(size: u8) -> TokenStream {
    let mut gin = quote! { };
    let mut tfor = quote! { };
    let mut def = quote! { };
    let mut undef = quote! { };
    let mut vals = quote! { };
    for i in 0..size {
        let ti = format_ident!("T{i}");
        gin = quote! { #gin #ti: AsField, };
        tfor = quote! { #tfor #ti, };
        def = quote! { #def D<#i, #ti>,};
        undef = quote! { #undef U<#i, #ti>,};
        vals = quote! { #vals U::<#i, _>(PhantomData), };
    }
    quote! { 
        impl<#gin> AsFlatProps for (#tfor ()) {
            type Defined = (#def);
            type Undefined = (#undef);
            fn as_flat_props() -> Self::Undefined {
                (#vals)
            }
        }
        impl<#gin V: JoinProps<(#undef)>, P: AsFlatProps<Undefined = V>> AsFlatProps for (#tfor P) {
            type Defined = V::DefinedResult;
            type Undefined = V::UndefinedResult;
            fn as_flat_props() -> Self::Undefined {
                V::join()
            }
        }
    }
}
// size = 3, shift = 2
// //   #gin                   #l               #r
// impl<T0, T1, T2> JoinProps<(U<0, T0>,)> for (U<0, T1>, U<1, T2>) {
// //                          #udef
//     type UndefinedResult = (U<0, T0>, U<1, T1>, U<2, T2>);
// //                        #def
//     type DefinedResult = (D<0, T0>, D<1, T1>, D<2, T2>);
//     fn join() -> Self::UndefinedResult {
// //       #res
//         (U::<0, _>(PhantomData), U::<1, _>(PhantomData), U::<2, _>(PhantomData))    
//     }
// }
fn impl_join_props(shift: u8, size: u8) -> TokenStream {
    let mut gin = quote! { };
    let mut l = quote! { };
    let mut r = quote! { };
    let mut def = quote! { };
    let mut undef = quote! { };
    let mut res = quote! { };
    for i in 0..size {
        let ti = format_ident!("T{i}");
        if i < size - shift {
            l = quote! { #l U<#i, #ti>, };
        }
        if i < shift {
            let ir = i + (size - shift);
            let tr = format_ident!("T{ir}");
            r = quote! { #r U<#i, #tr>, };
        }
        gin = quote! { #gin #ti, };
        res = quote! { #res U::<#i, _>(PhantomData), };
        def = quote! { #def D<#i, #ti>, };
        undef = quote! { #undef U<#i, #ti>, };
    }

    let dbg0 = format_ident!("join_props_shift_{shift}_size_{size}");
    quote! {
        fn #dbg0() { }
        impl<#gin> JoinProps<(#l)> for (#r) {
            type DefinedResult = (#def);
            type UndefinedResult = (#undef);
            fn join() -> Self::UndefinedResult {
                (#res)
            }
        }
    }
}


// impl<T0: DefinedValue, T1: DefinedValue, T2: DefinedValue> Props<(T0, T1, T2)> {
//     pub fn defined(self) -> Props<(D<0, T0::Value>, D<1, T1::Value>, D<2, T2::Value>)> {
//         let (p0,p1,p2) = self.0;
//         Props((
//             D::<0, _>(p0.extract_value()),
//             D::<1, _>(p1.extract_value()),
//             D::<2, _>(p2.extract_value()),
//         ))
//     }
// }
fn impl_defined(size: u8) -> TokenStream {
    let mut gin = quote! { };
    let mut gout = quote! { };
    let mut pout = quote! { };
    let mut dcstr = quote! { };
    let mut vals = quote! { };
    for i in 0..size {
        let ti = format_ident!("T{i}");
        let pi = format_ident!("p{i}");
        gin = quote! { #gin #ti: DefinedValue, };
        gout = quote! { #gout #ti, };
        pout = quote! { #pout D<#i, #ti::Value>, };
        dcstr = quote! { #dcstr #pi, };
        vals = quote! { #vals D::<#i, _>(#pi.extract_value()), }
    }
    quote! { 
        impl<#gin> Props<(#gout)> {
            pub fn defined(self) -> Props<(#pout)> {
                let (#dcstr) = self.0;
                Props((#vals))
            }
        }
    }
}
