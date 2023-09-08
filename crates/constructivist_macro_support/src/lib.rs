use syn::{parse_macro_input, DeriveInput, Data, Type, Expr, parse::Parse, bracketed, Token, Ident, parenthesized, spanned::Spanned};
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


#[proc_macro_derive(Construct, attributes(extends, mixin, required, default))]
pub fn derive_construct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match Construct::from_derive(input, ConstructMode::object()) {
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c
    };
    proc_macro::TokenStream::from(constructable.build(lib()))
}
#[proc_macro_derive(Mixin, attributes(required, default))]
pub fn derive_mixin(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let constructable = match Construct::from_derive(input, ConstructMode::mixin()) {
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
        Ok(c) => c
    };
    proc_macro::TokenStream::from(constructable.build(lib()))
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

enum PropDefault {
    None,
    Default,
    Custom(Expr)
}
struct Prop {
    name: Ident,
    ty: PropType,
    default: PropDefault,
}

impl Parse for Prop {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let mut default = PropDefault::None;
        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            default = PropDefault::Custom(input.parse()?);
        }
        Ok(Prop { name, ty, default })
    }
}

macro_rules! throw {
    ($loc:expr, $msg:expr) => {
        return Err(syn::Error::new($loc.span(), $msg));
    };
}

enum ConstructMode {
    Pure,
    Mixin,
    Object { extends: Option<Type>, mixins: Vec<Type> },
}

impl ConstructMode {
    fn pure() -> Self {
        ConstructMode::Pure
    }
    fn mixin() -> Self {
        ConstructMode::Mixin
    }
    fn object() -> Self {
        ConstructMode::Object { extends: None, mixins: vec![] }
    }
    fn is_pure(&self) -> bool {
        match self {
            ConstructMode::Pure => true,
            _ => false
        }
    }
    fn is_mixin(&self) -> bool {
        match self {
            ConstructMode::Mixin => true,
            _ => false
        }
    }
    fn is_object(&self) -> bool {
        match self {
            ConstructMode::Object { .. } => true,
            _ => false
        }
    }
    fn set_extends(&mut self, ty: Type) -> Result<(), syn::Error> {
        match self {
            ConstructMode::Object { extends, .. } => {
                *extends = Some(ty);
                Ok(())
            },
            _ => {
                throw!(ty, "set_extends(..) available only for ConstructMode::Object");
            }

        }
    }
    fn push_mixin(&mut self, ty: Type) -> Result<(), syn::Error> {
        match self {
            ConstructMode::Object { mixins, .. } => {
                // throw!(ty, format!("adding mixin for {:?}", ty.to_token_stream()));
                mixins.push(ty);
                Ok(())
            },
            _ => {
                throw!(ty, "push_mixin(..) available only for ConstructMode::Object");
            }

        }

    }
}

struct Construct {
    ty: Type,
    props: Vec<Prop>,
    body: Option<Expr>,
    mode: ConstructMode,
    // extends: Option<Type>,
}

struct Object {
    construct: Construct,
    extends: Option<Type>
}

impl Parse for Construct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;
        let mut extends = None;
        if let Ok(ident) = input.parse::<Ident>() {
            if &ident.to_string() != "extends" {
                return Err(syn::Error::new(ident.span(), "Expected `extends` ident"));
            }
            extends = Some(input.parse()?)
        }
        let mode = ConstructMode::Object { extends, mixins: vec![] };
        let content;
        parenthesized!(content in input);
        let props = content.parse_terminated(Prop::parse, Token![,])?;
        let props = props.into_iter().collect();
        let body = Some(input.parse()?);
        Ok(Construct { ty, props, body, mode })
    }
}

impl Construct {
    fn build(&self, lib: TokenStream) -> TokenStream {
        let ty = &self.ty;
        let Some(type_ident) = ty.as_ident() else {
            return quote!(compile_error!("Can't implement Construct for {}", stringify!(#ty)));
        };
        let mod_ident = format_ident!(                      // slider_construct
            "{}_construct",
            type_ident.to_string().to_lowercase()
        );
        let mut type_props = quote! { };                   // slider_construct::min, slider_construct::max, slider_construct::val,
        let mut type_props_deconstruct = quote! { };       // slider_construct::min(min), slider_construct::max(max), slider_construct::val(val),
        let mut prop_values = quote! { };                  // min, max, val,
        let mut impls = quote! { };
        let mut fields = quote! { };
        let mut fields_new = quote! { };
        for prop in self.props.iter() {
            let PropType::Single(prop_ty) = &prop.ty else {
                return quote!(compile_error!("Union props not supported yet."))
            };
            let ident = &prop.name;
            prop_values = quote! { #prop_values #ident, };
            type_props = quote! { #type_props #mod_ident::#ident, };
            type_props_deconstruct = quote! { #type_props_deconstruct #mod_ident::#ident(mut #ident), };
            fields = quote! { #fields
                #[allow(unused_variables)]
                pub #ident: #lib::Prop<#ident, #prop_ty>,
            };
            fields_new = quote! { #fields_new #ident: #lib::Prop(::std::marker::PhantomData), };
            let default = match &prop.default {
                PropDefault::Custom(default) => {
                    quote! { 
                        impl Default for #ident {
                            fn default() -> Self {
                                #ident(#default)
                            }
                        }
                    }
                },
                PropDefault::Default => {
                    quote! { 
                        impl Default for #ident {
                            fn default() -> Self {
                                #ident(Default::default())
                            }
                        }
                    }
                },
                PropDefault::None => {
                    quote! { }
                }
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
                impl #lib::New<#prop_ty> for #ident {
                    fn new(from: #prop_ty) -> #ident {
                        #ident(from)
                    }
                }
            };
        }
        let construct = if let Some(expr) = &self.body {
            expr.clone()
        } else {
            syn::parse2(quote!{ 
                Self { #prop_values }
            }).unwrap()
        };

        let object = if let ConstructMode::Object { extends, mixins } = &self.mode {
            let extends = if let Some(extends) = extends {
                quote! { #extends }
            } else {
                quote! { () }
            };

            let mut mixed_props = quote! { };
            let mut expanded_props = quote! { <Self::Extends as #lib::Object>::ExpandedProps };
            let mut hierarchy = quote! { <Self::Extends as #lib::Object>::Hierarchy };
            let mut deconstruct = quote! { };
            let mut construct = quote! { <Self::Extends as #lib::Object>::build(rest) };
            for mixin in mixins.iter().rev() {
                
                let mixin_params = if let Some(ident) = mixin.as_ident() {
                    format_ident!("{}_params", ident.to_string().to_lowercase())
                } else {
                    return quote!(compile_error!("Can't construct params ident"));
                };
                if mixed_props.is_empty() {
                    mixed_props = quote! { <#mixin as Construct>::Props, };
                    deconstruct = quote! { #mixin_params };
                } else {
                    mixed_props = quote! {  #lib::Mix<<#mixin as Construct>::Props, #mixed_props> };
                    deconstruct = quote! { (#mixin_params, #deconstruct) };
                }
                expanded_props = quote! { #lib::Mix<<#mixin as Construct>::Props, #expanded_props> };
                construct = quote! { ( #mixin::construct(#mixin_params), #construct ) };
                hierarchy = quote! { (#mixin, #hierarchy) };
            }
            let mixed_props = if mixed_props.is_empty() {
                quote! { (#type_props) }
            } else {
                quote! { #lib::Mix<(#type_props), #mixed_props> }
            };
            let deconstruct = if deconstruct.is_empty() {
                quote! { self_params }
            } else {
                quote! { (self_params, #deconstruct) }
            };
            let construct = quote! {
                (
                    <Self as #lib::Construct>::construct(self_params),
                    #construct
                )
            };
            
            quote! {
                impl #lib::Object for #type_ident {
                    type Extends = #extends;
                    type Fields = #mod_ident::Fields;
                    type Methods = #mod_ident::Methods;
                    type MixedProps = (#mixed_props);
                    // type Hierarchy =  (Self, <Self::Extends as #lib::Object>::Hierarchy);
                    type Hierarchy = (Self, #hierarchy);
                    // type ExpandedProps = #lib::Mix<(#type_props), <Self::Extends as #lib::Object>::ExpandedProps>;
                    type ExpandedProps = #lib::Mix<(#type_props), #expanded_props>;
                    
                    
                    fn build<P, const I: u8>(params: P) -> Self::Hierarchy where P: #lib::ExtractParams<
                        I, Self::MixedProps,
                        Value = <Self::MixedProps as #lib::Extractable>::Output,
                        Rest = <<<Self::Extends as #lib::Object>::ExpandedProps as #lib::Extractable>::Input as #lib::AsParams>::Defined
                    > {
                        let (#deconstruct, rest) = params.extract_params();
                        #construct
                        // let (args, rest) = params.extract_params();
                        // (
                        //     <Self as #lib::Construct>::construct(args),
                        //     <Self::Extends as #lib::Object>::build(rest)
                        // )
                    }
                }
            }
        } else {
            quote! { }
        };
        let mixin = if self.mode.is_mixin() {
            quote! { 
                impl #lib::Mixin for #type_ident {
                    type Fields<T: #lib::Singleton + 'static> = #mod_ident::Fields<T>;
                    type Methods<T: #lib::Singleton + 'static> = #mod_ident::Methods<T>;
                } 
            }
        } else {
            quote! { }
        };
        let decls = match &self.mode {
            ConstructMode::Object { extends, mixins } => {
                let extends = if let Some(extends) = extends {
                    quote! { #extends }
                } else {
                    quote! { () }
                };
                let mut deref_fields = quote! { <#extends as #lib::Object>::Fields };
                let mut deref_methods = quote! { <#extends as #lib::Object>::Methods };
                for mixin in mixins.iter() {
                    // throw!(mixin, "got mixin");
                    deref_fields = quote! { <#mixin as #lib::Mixin>::Fields<#deref_fields> };
                    deref_methods = quote! { <#mixin as #lib::Mixin>::Methods<#deref_methods> };
                }

                quote! {
                    pub struct Fields {
                        #fields
                    }
    
                    pub struct Methods;
                    impl #lib::Singleton for Fields {
                        fn instance() -> &'static Self {
                            &Fields {
                                #fields_new
                            }
                        }
                    }
                    impl #lib::Singleton for Methods {
                        fn instance() -> &'static Self {
                            &Methods
                        }
                    }
                    impl ::std::ops::Deref for Fields {
                        type Target = #deref_fields;
                        fn deref(&self) -> &Self::Target {
                            <#deref_fields as #lib::Singleton>::instance()
                        }
                    }
                    impl #lib::Methods<#ty> for Methods { }
                    impl ::std::ops::Deref for Methods {
                        type Target = #deref_methods;
                        fn deref(&self) -> &Self::Target {
                            <#deref_methods as #lib::Singleton>::instance()
                        }
                    }
    
                }
            },
            ConstructMode::Mixin => quote! {
                pub struct Fields<T: #lib::Singleton> {
                    #fields
                    __base__: ::std::marker::PhantomData<T>,
                }
                pub struct Methods<T: #lib::Singleton>(
                    ::std::marker::PhantomData<T>
                );
                impl<T: #lib::Singleton> #lib::Singleton for Fields<T> {
                    fn instance() -> &'static Self {
                        &Fields {
                            #fields_new
                            __base__: ::std::marker::PhantomData,
                        }
                    }
                }
                impl<T: #lib::Singleton> #lib::Singleton for Methods<T> {
                    fn instance() -> &'static Self {
                        &Methods(::std::marker::PhantomData)
                    }
                }
                impl<T: #lib::Singleton + 'static> std::ops::Deref for Fields<T> {
                    type Target = T;
                    fn deref(&self) -> &Self::Target {
                        T::instance()
                    }
                }
                impl<T: #lib::Singleton + 'static> std::ops::Deref for Methods<T> {
                    type Target = T;
                    fn deref(&self) -> &Self::Target {
                        T::instance()
                    }
                }
            },
            _ => quote! { }
        };
        quote! {
            mod #mod_ident {
                use super::*;
                #decls
                #impls
            }
            impl #lib::NonUnit for #type_ident { }
            impl #lib::Construct for #type_ident {
                type Props = ( #type_props );
                fn construct(props: Self::Props) -> Self {
                    let (#type_props_deconstruct) = props;
                    #construct
                }
            }
            #object
            #mixin

        }
    }


    pub fn from_derive(input: DeriveInput, mut mode: ConstructMode) -> Result<Self, syn::Error> {
        if input.generics.params.len() > 0 {
            throw!(input.ident, "#[derive(Construct)] doesn't support generics yet.");
        }
        let ident = input.ident.clone();                      // Slider
        let ty = syn::parse2(quote!{ #ident }).unwrap();
        if let Some(extends) = input.attrs.iter().find(|a| a.path().is_ident("extends")) {
            if !mode.is_object() {
                throw!(extends, "#[extends(..) only supported by #[derive(Object)].");
            }
            mode.set_extends(extends.parse_args()?)?
        }
        if let Some(mixin) = input.attrs.iter().find(|a| a.path().is_ident("mixin")) {
            // throw!(mixin, "found mixin");
            if !mode.is_object() {
                throw!(mixin, "#[mixin(..) only supported by #[derive(Object)].");
            }
            // mixin.meta.
            mixin.parse_nested_meta(|meta| {
                mode.push_mixin(syn::parse2(meta.path.into_token_stream())?)
                // for mixin in meta.input.parse_terminated(Type::parse, Token![,])?.iter() {
                //     throw!(mixin, "adding mixin");
                // }
                // Ok(())
            })?;
        }
    
        let Data::Struct(input) = input.data else {
            throw!(input.ident, "#[derive(Construct)] only supports named structs. You can use `constructable!` for complex cases.");
        };
        let mut props = vec![];
        for field in input.fields.iter() {
            let ty = PropType::Single(field.ty.clone());
            let Some(name) = field.ident.clone() else {
                throw!(field, "#[derive(Construct)] only supports named structs. You can use `constructable!` for complex cases.");
            };
            let default = if field.attrs.iter().any(|a| a.path().is_ident("required")) {
                PropDefault::None
            } else if let Some(default) = field.attrs.iter().find(|a| a.path().is_ident("default")) {
                let Ok(expr) = default.parse_args::<Expr>() else {
                    throw!(name, "Invalid expression for #[default(expr)].");
                };
                PropDefault::Custom(expr)
            } else {
                PropDefault::Default
            };
            props.push(Prop { ty, name, default });
        }
        let body = None;
        Ok(Construct {
            ty, props, body, mode,
        })
    }
}


#[proc_macro]
pub fn constructable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Construct);
    proc_macro::TokenStream::from(input.build(lib()))
}

#[proc_macro]
pub fn construct_implementations(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let max_size = 16;
    let extract_field_impls = impl_all_extract_field(max_size);
    let add_to_props = impl_all_add_to_props(max_size);
    let defined = impl_all_defined(max_size);
    let extracts = impl_all_extracts(max_size);
    let mixed = impl_all_mixed(max_size);
    let as_params = impl_all_as_params(max_size);
    proc_macro::TokenStream::from(quote! {
        #extract_field_impls
        #add_to_props
        #defined
        #extracts
        #as_params
        #mixed
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
fn impl_all_defined(max_size: u8) -> TokenStream {
    let mut out = quote! { };
    for size in 1..max_size + 1 {
        let defined = impl_defined(size);
        out = quote! { #out #defined }
    }
    out
}
fn impl_all_extracts(max_size: u8) -> TokenStream {
    let mut out = quote! { };
    for size in 1..max_size + 1 {
        let extractable = impl_extractable(size);
        out = quote! { #out #extractable };
        for defined in 0..size + 1{
            let extract = impl_extract(defined, size);
            out = quote! { #out #extract };
        }
    }
    out
}
fn impl_all_mixed(max_size: u8) -> TokenStream {
    let mut out = quote! { };
    for size in 1..max_size + 1 {
        for left in 0..size + 1 {
            let right = size - left;
            let mixed = impl_mixed(left, right);
            out = quote! { #out #mixed };
        }
    }
    out
}
/// ```ignore
/// impl<T0, T1> AsParams for (D<0, T0>, D<1, T1>) {
/// type Undefined = (U<0, T0>, U<1, T1>);
///     fn as_params() -> Props<Self::Undefined> {
///         Props((
///             U::<0, T0>(PhantomData),
///             U::<1, T1>(PhantomData)
///         ))
///     }
/// }
/// ```
fn impl_all_as_params(max_size: u8) -> TokenStream {
    let mut out = quote! { };
    for size in 0..max_size+1 {
        let mut ts = quote! { };
        let mut ds = quote! { };
        let mut us = quote! { };
        let mut ps = quote! { };
        for i in 0..size {
            let ti = format_ident!("T{i}");
            ts = quote! { #ts #ti, };
            ds = quote! { #ds D<#i, #ti>, };
            us = quote! { #us U<#i, #ti>, };
            ps = quote! { #ps U::<#i, #ti>(::std::marker::PhantomData), }
        }
        out = quote! { #out
            impl<#ts> AsParams for (#ds) {
                type Undefined = Props<(#us)>;
                type Defined = Props<(#ds)>;
                fn as_params() -> Self::Undefined {
                    Props(( #ps ))
                }
            }    
        }
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
//       #gin                                              #pundef
/// impl<T1, A0, A2, A3> std::ops::Add<D<1, T1>> for Props<(A0, U<1, T1>, A2, A3)> {
//                           #pout
///     type Output = Props<(A0, D<1, T1>, A2, A3)>;
///     fn add(self, rhs: D<1, T1>) -> Self::Output {
//               #dcs
///         let (p0, _, p2, p3) = self.0;
//                 #vls
///         Props((p0, rhs, p2, p3))
///     }
/// }
//       #gin                                              #pdef
/// impl<T1, A0, A2, A3> std::ops::Add<D<1, T1>> for Props<(A0, D<1, T1>, A2, A3)> {
//                           #pout
///     type Output = PropConflict<T1>;
///     fn add(self, _: D<1, T1>) -> Self::Output {
///         PropConflict::new()
///     }
/// }
/// ```
fn impl_add_to_props(idx: u8, size: u8) -> TokenStream {
    let ti = format_ident!("T{idx}");
    let di = quote! { D<#idx, #ti> };
    let ui = quote! { U<#idx, #ti> };
    let mut gin = quote! { };
    let mut pundef = quote! { };
    let mut pdef = quote! { };
    let mut pout = quote! { };
    let mut dcs = quote! { };
    let mut vls = quote! { };
    for i in 0..size {
        if i == idx {
            pundef = quote! { #pundef #ui, };
            pdef = quote! { #pdef #di, };
            pout = quote! { #pout #di, };
            dcs = quote! { #dcs _, };
            vls = quote! { #vls rhs, };
        } else {
            let ai = format_ident!("A{i}");
            let pi = format_ident!("p{i}");
            gin = quote! { #gin #ai, };
            pundef = quote! { #pundef #ai, };
            pdef = quote! { #pdef #ai, };

            pout = quote! { #pout #ai, };
            dcs = quote! { #dcs #pi, };
            vls = quote! { #vls #pi, };
        }
    }
    quote! {
        impl<#ti, #gin> std::ops::Add<#di> for Props<(#pundef)> {
            type Output = Props<(#pout)>;
            fn add(self, rhs: #di) -> Self::Output {
                let (#dcs) = self.0;
                Props((#vls))
            }
        }

        impl<#ti, #gin> std::ops::Add<#di> for Props<(#pdef)> {
            type Output = PropConflict<#ti>;
            fn add(self, _: #di) -> Self::Output {
                PropConflict::new()
            }
        }
    }
}


/// ```ignore
/// impl<T0, T1> Extractable for (T0, T1) {
///     type Input = (D<0, T0>, D<1, T1>);
///     type Output = (T0, T1);
///     fn extract(input: Self::Input) -> Self::Output {
///         let (p0, p1) = input;
///         (p0.0, p1.0)
///     }
/// }
/// impl<T0, T1, T2, T3, E: Extractable<Input = (T0, T1)>> ExtractParams<2, E> for Props<(T0, T1, T2, T3)> 
/// where
///     T2: Shift<0>,
///     T3: Shift<1>,
/// {
///     type Value = E::Output;
///     type Rest = Props<(T2::Target, T3::Target)>;
///     fn extract_params(self) -> (Self::Value, Self::Rest) {
///         let (p0, p1, p2, p3) = self.0;
///         (
///             E::extract((p0, p1)),
///             Props((p2.shift(), p3.shift()))
///         )
///     }
/// }
/// ```
fn impl_extractable(size: u8) -> TokenStream {
    let mut ein = quote! { };
    let mut edef = quote! { };
    let mut eout = quote! { };
    let mut dcstr = quote! { };

    for i in 0..size {
        let ti = format_ident!("T{i}");
        let pi = format_ident!("p{i}");
        ein = quote! { #ein #ti, };
        edef = quote! { #edef D<#i, #ti>, };
        dcstr = quote! { #dcstr #pi, };
        eout = quote! { #eout #pi.0, };
    }
    quote! {
        impl<#ein> Extractable for (#ein) {
            type Input = (#edef);
            type Output = (#ein);
            fn extract(input: Self::Input) -> Self::Output {
                let (#dcstr) = input;
                (#eout)
            }
        }
    }
}
fn impl_extract(defined: u8, size: u8) -> TokenStream {
    let mut ein = quote! { };
    let mut pin = quote! { };
    let mut pfor = quote! { };
    let mut pcstr = quote! { };
    let mut trest = quote! { };
    let mut pdcstr = quote! { };
    let mut pout = quote! { };
    let mut pprops = quote! { };

    for i in 0..size {
        let ti = format_ident!("T{i}");
        let pi = format_ident!("p{i}");
        if i < defined {
            ein = quote! { #ein #ti, };
            pout = quote! { #pout #pi, }
        } else {
            let j = i - defined;
            pcstr = quote! { #pcstr #ti: Shift<#j>, };
            trest = quote! { #trest #ti::Target, };
            pprops = quote! { #pprops #pi.shift(), };
        }
        pin = quote! { #pin #ti, };
        pfor = quote! { #pfor #ti, };
        pdcstr = quote! { #pdcstr #pi, };
    }
    quote! {
        impl<#pin E: Extractable<Input = (#ein)>> ExtractParams<#defined, E> for Props<(#pin)> 
        where #pcstr
        {
            type Value = E::Output;
            type Rest = Props<(#trest)>;
            fn extract_params(self) -> (Self::Value, Self::Rest) {
                let (#pdcstr) = self.0;
                (
                    E::extract((#pout)),
                    Props((#pprops))
                )
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


/// ```ignore
/// impl<L0, R0, R1> Mixed<(D<0, R0>, D<1, R1>)> for (D<0, L0>,) {
///     type Output = (D<0, L0>, D<1, R0>, D<2, R1>);
///     fn split(joined: Self::Output) -> (Self, (D<0, R0>, D<1, R1>)) {
///         let (l0, r0, r1) = joined;
///         let r0 = D::<0, _>(r0.0);
///         let r1 = D::<1, _>(r1.0);
///         ((l0,), (r0, r1))
///         
///     }
/// }
/// ```
fn impl_mixed(left: u8, right: u8) -> TokenStream {
    let mut ls = quote! { };        // L0,
    let mut rs = quote! { };        // R0, R1,
    let mut dls = quote! { };       // D<0, L0>,
    let mut drs = quote! { };       // D<0, R0>, D<1, R1>,
    let mut lvs = quote! { };       // l0,
    let mut rvs = quote! { };       // r0, r1,
    let mut shift = quote! { };     // let r0 = D::<0, _>(r0.0);
                                    // let r1 = D::<1, _>(r1.0);
    let mut output = quote! { };    // D<0, L0>, D<1, R0>, D<2, R1>
    for i in 0..left.max(right) {
        let li = format_ident!("L{i}");
        let ri = format_ident!("R{i}");
        let lv = format_ident!("l{i}");
        let rv = format_ident!("r{i}");
        if i < left {
            ls = quote! { #ls #li, };
            dls = quote! { #dls D<#i, #li>, };
            lvs = quote! { #lvs #lv, };
        }
        if i < right {
            rs = quote! { #rs #ri, };
            drs = quote! { #drs D<#i, #ri>, };
            rvs = quote! { #rvs #rv, };
            shift = quote! { #shift let #rv = D::<#i, _>(#rv.0); }
        }
    }
    for i in 0..left+right {
        let ti = if i < left {
            format_ident!("L{i}")
        } else {
            format_ident!("R{}", i - left)
        };
        output = quote! { #output D<#i, #ti>, };
    }
    quote! {
        impl<#ls #rs> Mixed<(#drs)> for (#dls) {
            type Output = (#output);
            fn split(joined: Self::Output) -> (Self, (#drs)) {
                let (#lvs #rvs) = joined;
                #shift
                ((#lvs), (#rvs))
            }
        }
    }
}
