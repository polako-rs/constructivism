use crate::context::Context;
use crate::exts::TypeExt;
use crate::throw;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    bracketed, parenthesized, parse::{Parse, ParseStream}, spanned::Spanned, Data, DeriveInput, Expr, Ident,
    Token, Type, Field, Fields,
};

enum ParamType {
    Single(Type),
    Union(Vec<Param>),
}
impl Parse for ParamType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let content;
            bracketed!(content in input);
            let params = content.parse_terminated(Param::parse, Token![,])?;
            Ok(ParamType::Union(params.into_iter().collect()))
        } else {
            Ok(ParamType::Single(input.parse()?))
        }
    }
}

enum ParamDefault {
    None,
    Default,
    Custom(Expr),
}
struct Param {
    name: Ident,
    ty: ParamType,
    default: ParamDefault,
}

impl Parse for Param {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let mut default = ParamDefault::None;
        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            default = ParamDefault::Custom(input.parse()?);
        }
        Ok(Param { name, ty, default })
    }
}

pub struct BuildedParams {
    // slider_construct::min, slider_construct::max, slider_construct::val,
    type_params: TokenStream,
    // slider_construct::min(min), slider_construct::max(max), slider_construct::val(val),
    type_params_deconstruct: TokenStream,
    // min, max, val,
    param_values: TokenStream,
    impls: TokenStream,
    fields: TokenStream,
    fields_new: TokenStream,
}
pub trait Params: Sized {
    fn from_fields(fields: &syn::Fields, name: &str, alter: &str) -> syn::Result<Self>;
    fn build(&self, ctx: &Context, ty: &Type, mod_ident: &Ident) -> syn::Result<BuildedParams>;
}
impl Params for Vec<Param> {
    fn from_fields(fields: &syn::Fields, name: &str, alter: &str) -> syn::Result<Self> {
        let mut params = vec![];
        for field in fields.iter() {
            let ty = ParamType::Single(field.ty.clone());
            let Some(name) = field.ident.clone() else {
                throw!(field, "#[derive({})] only supports named structs. You can use `{}!` for complex cases.", name, alter);
            };
            let default = if field.attrs.iter().any(|a| a.path().is_ident("required")) {
                ParamDefault::None
            } else if let Some(default) = field.attrs.iter().find(|a| a.path().is_ident("default"))
            {
                let Ok(expr) = default.parse_args::<Expr>() else {
                    throw!(name, "Invalid expression for #[default(expr)].");
                };
                ParamDefault::Custom(expr)
            } else {
                ParamDefault::Default
            };
            params.push(Param { ty, name, default });
        }
        Ok(params)
    }

    fn build(&self, ctx: &Context, ty: &Type, mod_ident: &Ident) -> syn::Result<BuildedParams> {
        let lib = ctx.path("constructivism");
        let mut type_params = quote! {}; // slider_construct::min, slider_construct::max, slider_construct::val,
        let mut type_params_deconstruct = quote! {}; // slider_construct::min(min), slider_construct::max(max), slider_construct::val(val),
        let mut param_values = quote! {}; // min, max, val,
        let mut impls = quote! {};
        let mut fields = quote! {};
        let mut fields_new = quote! {};
        for param in self.iter() {
            let ParamType::Single(param_ty) = &param.ty else {
                throw!(ty, "Union params not supported yet.");
            };
            let ident = &param.name;
            param_values = quote! { #param_values #ident, };
            type_params = quote! { #type_params #mod_ident::#ident, };
            type_params_deconstruct =
                quote! { #type_params_deconstruct #mod_ident::#ident(mut #ident), };
            fields = quote! { #fields
                #[allow(unused_variables)]
                pub #ident: #lib::Param<#ident, #param_ty>,
            };
            fields_new = quote! { #fields_new #ident: #lib::Param(::std::marker::PhantomData), };
            let default = match &param.default {
                ParamDefault::Custom(default) => {
                    quote! {
                        impl Default for #ident {
                            fn default() -> Self {
                                #ident(#default)
                            }
                        }
                    }
                }
                ParamDefault::Default => {
                    quote! {
                        impl Default for #ident {
                            fn default() -> Self {
                                #ident(Default::default())
                            }
                        }
                    }
                }
                ParamDefault::None => {
                    quote! {}
                }
            };
            impls = quote! { #impls
                #default
                #[allow(non_camel_case_types)]
                pub struct #ident(pub #param_ty);
                impl<T: Into<#param_ty>> From<T> for #ident {
                    fn from(__value__: T) -> Self {
                        #ident(__value__.into())
                    }
                }
                impl #lib::AsField for #ident {
                    fn as_field() -> #lib::Field<Self> {
                        #lib::Field::new()
                    }
                }
                impl #lib::New<#param_ty> for #ident {
                    fn new(from: #param_ty) -> #ident {
                        #ident(from)
                    }
                }
            };
        }
        Ok(BuildedParams {
            type_params,
            type_params_deconstruct,
            param_values,
            impls,
            fields,
            fields_new,
        })
    }
}

pub struct Sequence {
    this: Type,
    segments: Vec<Type>,
    next: Type,
}

impl Parse for Sequence {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let this = input.parse()?;
        input.parse::<Token![->]>()?;
        let mut next = input.parse()?;
        let mut segments = vec![];
        while input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            segments.push(next);
            next = input.parse()?;
        }
        Ok(Sequence {
            this,
            segments,
            next,
        })
    }
}

impl Sequence {
    pub fn from_derive(input: &DeriveInput) -> syn::Result<Self> {
        let attrs = input
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("construct"))
            .collect::<Vec<_>>();
        if attrs.len() == 0 {
            throw!(input.ident, "Missing #[construct(..) attribute");
        }
        if attrs.len() > 1 {
            throw!(attrs[1], "Unexpected #[construct(..) attribute");
        }
        Ok(attrs[0].parse_args()?)
    }
}

pub struct DeriveSegment {
    ty: Type,
    params: Vec<Param>,
    props: Props,
    body: Option<Expr>,
}

impl Parse for DeriveSegment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;
        let content;
        parenthesized!(content in input);
        let params = content.parse_terminated(Param::parse, Token![,])?;
        let params = params.into_iter().collect();
        let body = Some(input.parse()?);
        Ok(DeriveSegment { ty, params, body, props: Props::default() })
    }
}

impl DeriveSegment {
    pub fn from_derive(input: DeriveInput) -> syn::Result<Self> {
        if input.generics.params.len() > 0 {
            throw!(
                input.ident,
                "#[derive(Segment)] doesn't support generics yet."
            );
        }
        let ty = &input.ident;
        let ty = syn::parse2(quote! { #ty })?;
        let Data::Struct(input) = input.data else {
            throw!(input.ident, "#[derive(Segment)] only supports named structs. You can use `derive_segment!` for complex cases.");
        };
        let params = Params::from_fields(&input.fields, "Segment", "derive_segment")?;
        let body = None;
        let props = Props::from_fields(&input.fields)?;
        Ok(DeriveSegment { ty, params, props, body })
    }

    pub fn build(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let ty = &self.ty;
        let lib = ctx.path("constructivism");
        let type_ident = ty.as_ident()?;
        let mod_ident = format_ident!(
            // input_segment
            "{}_segment",
            type_ident.to_string().to_lowercase()
        );
        let design = format_ident!("{type_ident}Design");
        let BuildedParams {
            fields,
            fields_new,
            impls,
            param_values,
            type_params,
            type_params_deconstruct,
        } = self.params.build(ctx, &ty, &mod_ident)?;
        let props_getters = self.props.build_lookup_getters(ctx, &ty)?;
        let props_setters = self.props.build_lookup_setters(ctx, &ty)?;
        let getters = self.props.build_getters(ctx)?;
        let setters = self.props.build_setters(ctx)?;
        let construct = if let Some(expr) = &self.body {
            expr.clone()
        } else {
            syn::parse2(quote! {
                Self { #param_values }
            })
            .unwrap()
        };
        let decls = quote! {

            // fields

            pub struct Fields<T: #lib::Singleton> {
                #fields
                __base__: ::std::marker::PhantomData<T>,
            }
            impl<T: #lib::Singleton> #lib::Singleton for Fields<T> {
                fn instance() -> &'static Self {
                    &Fields {
                        #fields_new
                        __base__: ::std::marker::PhantomData,
                    }
                }
            }
            impl<T: #lib::Singleton + 'static> std::ops::Deref for Fields<T> {
                type Target = T;
                fn deref(&self) -> &Self::Target {
                    T::instance()
                }
            }

            // Props
            pub struct Props<M, T: #lib::Props<M>>(
                ::std::marker::PhantomData<(M, T)>,
            );
            pub struct Getters<'a>(&'a #ty);
            pub struct Setters<'a>(&'a mut #ty);
            impl<'a> #lib::Getters<'a, #ty> for Getters<'a> {
                fn from_ref(from: &'a #ty) -> Self {
                    Self(from)
                }
                fn into_value(self) -> #lib::Value<'a, #ty> {
                    #lib::Value::Ref(&self.0)
                }
            }
            impl<'a> #lib::Setters<'a, #ty> for Setters<'a> {
                fn from_mut(from: &'a mut #ty) -> Self {
                    Self(from)
                }
            }
            impl<T> Props<#lib::Lookup, T> 
            where T:
                #lib::Props<#lib::Lookup> +
                #lib::Props<#lib::Get> +
                #lib::Props<#lib::Set>
            {
                pub fn getters(&self) -> &'static Props<#lib::Get, T> {
                    <Props<#lib::Get, T> as #lib::Singleton>::instance()
                }
                pub fn setters(&self) -> &'static Props<#lib::Set, T> {
                    <Props<#lib::Set, T> as #lib::Singleton>::instance()
                }
            }
            impl<T: #lib::Props<#lib::Get>> Props<#lib::Get, T> {
                #props_getters
            }
            impl<T: #lib::Props<#lib::Set>> Props<#lib::Set, T> {
                #props_setters
            }
            impl<'a> Getters<'a> {
                #getters
            }
            impl<'a> Setters<'a> {
                #setters
            }
            impl<M: 'static, T: #lib::Props<M> + 'static> std::ops::Deref for Props<M, T> {
                type Target = T;
                fn deref(&self) -> &Self::Target {
                    <T as #lib::Singleton>::instance()
                }
            }
            impl<M, T: #lib::Props<M>> #lib::Singleton for Props<M, T> {
                fn instance() -> &'static Self {
                    &Props(::std::marker::PhantomData)
                }
            }
            impl<M, T: #lib::Props<M>> #lib::Props<M> for Props<M, T> { }


        };
        Ok(quote! {
            mod #mod_ident {
                use super::*;
                #decls
                #impls
            }
            impl #lib::ConstructItem for #type_ident {
                type Params = ( #type_params );
                type Getters<'a> = #mod_ident::Getters<'a>;
                type Setters<'a> = #mod_ident::Setters<'a>;
                fn construct_item(params: Self::Params) -> Self {
                    let (#type_params_deconstruct) = params;
                    #construct
                }
            }
            impl #lib::Segment for #type_ident {
                type Props<M, T: #lib::Props<M> + 'static> = #mod_ident::Props<M, T>;
                type Fields<T: #lib::Singleton + 'static> = #mod_ident::Fields<T>;
                type Design<T: #lib::Singleton + 'static> = #design<T>;
            }
            pub struct #design<T>(
                ::std::marker::PhantomData<T>
            );
            impl<T: #lib::Singleton> #lib::Singleton for #design<T> {
                fn instance() -> &'static Self {
                    &#design(::std::marker::PhantomData)
                }
            }
            impl<T: #lib::Singleton + 'static> std::ops::Deref for #design<T> {
                type Target = T;
                fn deref(&self) -> &Self::Target {
                    T::instance()
                }
            }
        })
    }
}

pub enum Prop {
    Construct { ident: Ident, ty: Type },
    Value { ident: Ident, ty: Type },
    GetSet { ident: Ident, get: Ident, set: Ident, ty: Type, },
}
impl Prop {
    pub fn from_field(field: &Field) -> syn::Result<Self> {
        let ty = field.ty.clone();
        let Some(ident) = field.ident.clone() else {
            throw!(field, "Anonymous fields not supported.");
        };
        let mut attrs = field.attrs.iter().filter(|a| a.path().is_ident("prop"));
        if let Some(attr) = attrs.next() {
            attr.parse_args_with(|input: ParseStream| {
                let spec = input.parse::<Ident>()?;
                if input.peek(Token![,]) {
                    let get = spec;
                    input.parse::<Token![,]>()?;
                    let set = input.parse()?;
                    return Ok(Prop::GetSet { ty, ident, get, set });
                }
                if &spec.to_string() == "construct" {
                    Ok(Prop::Construct { ident, ty })
                } else {
                    throw!(spec, "Expected construct, skip or get, set");
                }
            })
        } else if let Some(ident) = field.ident.clone() {
            Ok(Prop::Value { ty, ident })
        } else {
            throw!(field, "#[param(get, set)] is required for unnamed struct fields");
        }
    }

    pub fn build_getter(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let lib = ctx.constructivism();
        Ok(match self {
            Prop::Value { ident, ty } => quote! {
                pub fn #ident(self) -> #lib::Value<'a, #ty> {
                    #lib::Value::Ref(&self.0.#ident)
                }
            },
            Prop::Construct { ident, ty } => quote! {
                pub fn #ident(self) -> <#ty as #lib::ConstructItem>::Getters<'a> {
                    <<#ty as #lib::ConstructItem>::Getters<'a> as #lib::Getters<'a, #ty>>::from_ref(
                        &self.0.#ident
                    )
                }
            },
            Prop::GetSet { ident, get, ty, .. } => quote! {
                pub fn #ident(self) -> #lib::Value<'a, #ty> {
                    #lib::Value::Val(self.0.#get())
                }
            },
        })
    }
    pub fn build_setter(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let lib = ctx.path("constructivism");
        Ok(match self {
            Prop::Value { ident, ty } => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    pub fn #setter(self, value: #ty) {
                        self.0.#ident = value;
                    }
                }
            },
            Prop::Construct { ident, ty } => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    pub fn #ident(self) -> <#ty as #lib::ConstructItem>::Setters<'a> {
                        <<#ty as #lib::ConstructItem>::Setters<'a> as #lib::Setters<'a, #ty>>::from_mut(
                            &mut self.0.#ident
                        )
                    }
                    pub fn #setter(self, value: #ty) {
                        self.0.#ident = value;
                    }
                }
            },
            Prop::GetSet { ident, set, ty, .. } => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    pub fn #ident(self, value: #ty) {
                        self.0.#set(value);
                    }
                    pub fn #setter(self, value: #ty) {
                        self.0.#set(value);
                    }
                }
            },
        })
    }

    pub fn build_lookup_getter(&self, ctx: &Context, this: &Type) -> syn::Result<TokenStream> {
        let lib = ctx.constructivism();
        Ok(match self {
            Prop::Value { ident, ty } => quote! {
                pub fn #ident<'a>(&self, this: &'a #this) -> #lib::Value<'a, #ty> {
                    #lib::Value::Ref(&this.#ident)
                }
            },
            Prop::Construct { ident, ty } => quote! {
                pub fn #ident<'a>(&self, this: &'a #this) -> <#ty as #lib::ConstructItem>::Getters<'a> {
                    <<#ty as #lib::ConstructItem>::Getters<'a> as #lib::Getters<'a, #ty>>::from_ref(
                        &this.#ident
                    )
                }

            },
            Prop::GetSet { ident, get, ty, .. } => quote! {
                pub fn #ident<'a>(&self, this: &'a #this) -> #lib::Value<'a, #ty> {
                    #lib::Value::Val(this.#get())
                }
            },
        })
    }

    pub fn build_lookup_setter(&self, ctx: &Context, this: &Type) -> syn::Result<TokenStream> {
        let lib = ctx.constructivism();
        Ok(match self {
            Prop::Value { ident, ty } => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    pub fn #ident(&self, this: &mut #this, value: #ty) {
                        this.#ident = value;
                    }
                    pub fn #setter(&self, this: &mut #this, value: #ty) {
                        this.#ident = value;
                    }
                }
            },
            Prop::Construct { ident, ty } => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    pub fn #ident<'a>(&self, this: &'a mut #this) -> <#ty as #lib::ConstructItem>::Setters<'a> {
                        <<#ty as #lib::ConstructItem>::Setters<'a> as #lib::Setters<'a, #ty>>::from_mut(
                            &mut this.#ident
                        )
                    }
                    pub fn #setter(&self, this: &mut #this, value: #ty) {
                        this.#ident = value;
                    }
                }
            },
            Prop::GetSet { ident, set, ty, .. } => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    pub fn #ident(self, this: &mut #this, value: #ty) {
                        this.#set(value);
                    }
                    pub fn #setter(self, this: &mut #this, value: #ty) {
                        this.#set(value);
                    }
                }
            },
        })
    }
}

#[derive(Default)]
pub struct Props(Vec<Prop>);
impl Props {
    pub fn from_fields(fields: &Fields) -> syn::Result<Self> {
        let mut props = vec![];
        for field in fields.iter() {
        
            if let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("prop")) {
                let skip = attr.parse_args_with(|input: ParseStream| {
                    if let Ok(ident) = input.parse::<Ident>() {
                        if &ident.to_string() == "skip" && input.is_empty() {
                            return Ok(true)
                        }
                    }
                    Ok(false)
                })?;
                if skip {
                    continue;
                }
            }
            props.push(Prop::from_field(field)?)
        }
        Ok(Props(props))
    }
    pub fn build_getters(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let mut out = quote! { };
        for prop in self.iter() {
            let getter = prop.build_getter(ctx)?;
            out = quote! { #out #getter }
        }
        Ok(out)
    }
    pub fn build_setters(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let mut out = quote! { };
        for prop in self.iter() {
            let setter = prop.build_setter(ctx)?;
            out = quote! { #out #setter }
        }
        Ok(out)
    }

    pub fn build_lookup_getters(&self, ctx: &Context, this: &Type) -> syn::Result<TokenStream> {
        let mut out = quote! { };
        for prop in self.iter() {
            let getter = prop.build_lookup_getter(ctx, this)?;
            out = quote! { #out #getter }
        }
        Ok(out)
    }

    pub fn build_lookup_setters(&self, ctx: &Context, this: &Type) -> syn::Result<TokenStream> {
        let mut out = quote! { };
        for prop in self.iter() {
            let setter = prop.build_lookup_setter(ctx, this)?;
            out = quote! { #out #setter }
        }
        Ok(out)
    }

    
}
impl std::ops::Deref for Props {
    type Target = Vec<Prop>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Props {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct DeriveConstruct {
    ty: Type,
    sequence: Sequence,
    params: Vec<Param>,
    props: Props,
    body: Option<Expr>,
}

impl Parse for DeriveConstruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let sequence: Sequence = input.parse()?;
        let ty = sequence.this.clone();
        let content;
        parenthesized!(content in input);
        let params = content.parse_terminated(Param::parse, Token![,])?;
        let params = params.into_iter().collect();
        let body = Some(input.parse()?);
        Ok(DeriveConstruct {
            ty,
            params,
            body,
            sequence,
            props: Props::default(),
        })
    }
}

impl DeriveConstruct {
    pub fn from_derive(input: DeriveInput) -> syn::Result<Self> {
        if input.generics.params.len() > 0 {
            throw!(
                input.ident,
                "#[derive(Construct)] doesn't support generics yet."
            );
        }
        let ident = input.ident.clone(); // Slider
        let ty = syn::parse2(quote! { #ident }).unwrap();
        let sequence = Sequence::from_derive(&input)?;
        let Data::Struct(input) = input.data else {
            throw!(input.ident, "#[derive(Construct)] only supports named structs. You can use `derive_construct!` for complex cases.");
        };
        let params = Params::from_fields(&input.fields, "Construct", "derive_construct")?;
        let props = Props::from_fields(&input.fields)?;
        let body = None;
        Ok(DeriveConstruct {
            ty,
            sequence,
            params,
            props,
            body,
        })
    }

    pub fn build(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let ty = &self.ty;
        let lib = ctx.path("constructivism");
        let type_ident = ty.as_ident()?;
        let mod_ident = format_ident!(
            // slider_construct
            "{}_construct",
            type_ident.to_string().to_lowercase()
        );
        let design = format_ident!("{}Design", type_ident.to_string());
        let mut deref_design;
        let BuildedParams {
            fields,
            fields_new,
            impls,
            param_values,
            type_params,
            type_params_deconstruct,
        } = self.params.build(ctx, &ty, &mod_ident)?;
        let props_getters = self.props.build_lookup_getters(ctx, &ty)?;
        let props_setters = self.props.build_lookup_setters(ctx, &ty)?;
        let getters = self.props.build_getters(ctx)?;
        let setters = self.props.build_setters(ctx)?;
        let decls = {
            let base = &self.sequence.next;
            let base = if !base.is_nothing() {
                quote! { #base }
            } else {
                quote! { () }
            };
            let mut deref_fields = quote! { <#base as #lib::Construct>::Fields };
            let mut deref_props = quote! { <#base as #lib::Construct>::Props<M> };
            deref_design = quote! { <#base as #lib::Construct>::Design };
            for segment in self.sequence.segments.iter() {
                deref_fields = quote! { <#segment as #lib::Segment>::Fields<#deref_fields> };
                deref_design = quote! { <#segment as #lib::Segment>::Design<#deref_design> };
                deref_props = quote! { <#segment as #lib::Segment>::Props<M, #deref_props> };
            }

            quote! {
                // Fields
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
                impl ::std::ops::Deref for Fields {
                    type Target = #deref_fields;
                    fn deref(&self) -> &Self::Target {
                        <#deref_fields as #lib::Singleton>::instance()
                    }
                }
                
                // Props
                pub struct Props<M>(::std::marker::PhantomData<M>);
                pub struct Getters<'a>(&'a #ty);
                pub struct Setters<'a>(&'a mut #ty);
                impl<'a> #lib::Getters<'a, #ty> for Getters<'a> {
                    fn from_ref(from: &'a #ty) -> Self {
                        Self(from)
                    }
                    fn into_value(self) -> #lib::Value<'a, #ty> {
                        #lib::Value::Ref(&self.0)
                    }
                }
                impl<'a> #lib::Setters<'a, #ty> for Setters<'a> {
                    fn from_mut(from: &'a mut #ty) -> Self {
                        Self(from)
                    }
                }
                impl Props<#lib::Lookup> {
                    pub fn getters(&self) -> &'static Props<#lib::Get> {
                        <Props<#lib::Get> as #lib::Singleton>::instance()
                    }
                    pub fn setters(&self) -> &'static Props<#lib::Set> {
                        <Props<#lib::Set> as #lib::Singleton>::instance()
                    }
                }
                impl Props<#lib::Get> {
                    #props_getters
                }
                impl Props<#lib::Set> {
                    #props_setters
                }
                impl<'a> Getters<'a> {
                    #getters
                }
                impl<'a> Setters<'a> {
                    #setters
                }
                impl<M: 'static> std::ops::Deref for Props<M> {
                    type Target = #deref_props;
                    fn deref(&self) -> &Self::Target {
                        <#deref_props as #lib::Singleton>::instance()
                    }
                }
                impl<M> #lib::Singleton for Props<M> {
                    fn instance() -> &'static Self {
                        &Props(::std::marker::PhantomData)
                    }
                }
                impl<M> #lib::Props<M> for Props<M> { }
            }
        };
        let derive = {
            if self.sequence.this.to_token_stream().to_string() != ty.to_token_stream().to_string()
            {
                throw!(self.sequence.this, "Seqence head doesn't match struct name");
            }
            let this = &self.sequence.this;
            let base = &self.sequence.next;
            let base = if !base.is_nothing() {
                quote! { #base }
            } else {
                quote! { () }
            };

            let mut mixed_params = quote! {};
            let mut expanded_params = quote! { <Self::Base as #lib::Construct>::ExpandedParams };
            let mut base_sequence = quote! { <Self::Base as #lib::Construct>::NestedSequence };
            let mut deconstruct = quote! {};
            let mut construct = quote! { <Self::Base as #lib::Construct>::construct(rest) };
            for segment in self.sequence.segments.iter().rev() {
                let segment_params =
                    format_ident!("{}_params", segment.as_ident()?.to_string().to_lowercase());
                if mixed_params.is_empty() {
                    mixed_params = quote! { <#segment as #lib::ConstructItem>::Params, };
                    deconstruct = quote! { #segment_params };
                } else {
                    mixed_params = quote! {  #lib::Mix<<#segment as #lib::ConstructItem>::Params, #mixed_params> };
                    deconstruct = quote! { (#segment_params, #deconstruct) };
                }
                expanded_params = quote! { #lib::Mix<<#segment as #lib::ConstructItem>::Params, #expanded_params> };
                construct = quote! { ( <#segment as #lib::ConstructItem>::construct_item(#segment_params), #construct ) };
                base_sequence = quote! { (#segment, #base_sequence) };
            }
            let mixed_params = if mixed_params.is_empty() {
                quote! { (#type_params) }
            } else {
                quote! { #lib::Mix<(#type_params), #mixed_params> }
            };
            let deconstruct = if deconstruct.is_empty() {
                quote! { self_params }
            } else {
                quote! { (self_params, #deconstruct) }
            };
            let construct = quote! {
                (
                    <Self as #lib::ConstructItem>::construct_item(self_params),
                    #construct
                )
            };
            quote! {
                impl #lib::Construct for #type_ident {
                    type Sequence = <Self::NestedSequence as #lib::Flattern>::Output;
                    type Base = #base;
                    type Fields = #mod_ident::Fields;
                    type Props<M> = #mod_ident::Props<M>;
                    type Design = #design;
                    type MixedParams = (#mixed_params);
                    type NestedSequence = (Self, #base_sequence);
                    type ExpandedParams = #lib::Mix<(#type_params), #expanded_params>;

                    fn construct<P, const I: u8>(params: P) -> Self::NestedSequence where P: #lib::ExtractParams<
                        I, Self::MixedParams,
                        Value = <Self::MixedParams as #lib::Extractable>::Output,
                        Rest = <<<Self::Base as #lib::Construct>::ExpandedParams as #lib::Extractable>::Input as #lib::AsParams>::Defined
                    > {
                        let _: Option<#this> = None;
                        let (#deconstruct, rest) = params.extract_params();
                        #construct
                    }
                }
            }
        };
        let construct = if let Some(expr) = &self.body {
            expr.clone()
        } else {
            syn::parse2(quote! {
                Self { #param_values }
            })
            .unwrap()
        };
        Ok(quote! {
            mod #mod_ident {
                use super::*;
                #decls
                #impls
            }
            impl #lib::ConstructItem for #type_ident {
                type Params = ( #type_params );
                type Getters<'a> = #mod_ident::Getters<'a>;
                type Setters<'a> = #mod_ident::Setters<'a>;
                fn construct_item(params: Self::Params) -> Self {
                    let (#type_params_deconstruct) = params;
                    #construct
                }
            }
            pub struct #design;
                impl #lib::Singleton for #design {
                    fn instance() -> &'static Self {
                        &#design
                    }
                }
                impl ::std::ops::Deref for #design {
                    type Target = #deref_design;
                    fn deref(&self) -> &Self::Target {
                        <#deref_design as #lib::Singleton>::instance()
                    }
                }
            #derive
        })
    }
}
