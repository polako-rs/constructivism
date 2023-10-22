use std::collections::HashMap;

use crate::context::Context;
use crate::exts::*;
use crate::throw;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    Attribute, Data, DeriveInput, Expr, Field, Fields, Ident, Token, Type, parse_quote,
};

pub struct Declarations {
    span: Span,
    decls: HashMap<String, TokenStream>,
}

impl Parse for Declarations {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let mut decls = HashMap::new();
        while let Ok(ident) = input.parse::<Ident>() {
            input.parse::<Token![=>]>()?;
            let mut stream = quote! {};
            while !input.peek(Token![;]) {
                let tt = input.parse::<TokenTree>()?;
                stream = quote! { #stream #tt };
            }
            input.parse::<Token![;]>()?;
            decls.insert(ident.to_string(), stream);
        }
        Ok(Declarations { decls, span })
    }
}

impl Declarations {
    pub fn parse_declaration<T: Parse>(&self, key: &str) -> syn::Result<T> {
        let Some(stream) = self.decls.get(key) else {
            throw!(self.span, "Missing {} declaration key", key);
        };
        parse2(stream.clone())
    }
    pub fn parse_or_default<T: Parse + Default>(&self, key: &str) -> syn::Result<T> {
        if let Some(stream) = self.decls.get(key) {
            parse2(stream.clone())
        } else {
            Ok(T::default())
        }
    }
}

pub enum ParamType {
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

pub enum ParamKind {
    Common,
    Required,
    Default(Expr),
    Skip(Expr)
}
impl Parse for ParamKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if &ident.to_string() == "required" {
            Ok(ParamKind::Required)
        } else if &ident.to_string() == "default" {
            input.parse::<Token![=]>()?;
            Ok(ParamKind::Default(input.parse()?))
        } else if &ident.to_string() == "skip" {
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                Ok(ParamKind::Skip(input.parse()?))
            } else {
                Ok(ParamKind::Skip(parse_quote!(Default::default())))
            }
        } else {
            throw!(ident, "Expected required|default|skip");
        }
    }
}

pub struct Param {
    pub name: Ident,
    pub ty: ParamType,
    pub kind: ParamKind,
    pub docs: Vec<Attribute>,
}

impl Parse for Param {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let docs = Attribute::parse_outer(&input)?
            .iter()
            .filter(|a| a.path().is_ident("doc"))
            .cloned()
            .collect();
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let kind = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            ParamKind::Default(input.parse()?)
        } else {
            ParamKind::Required
        };
        Ok(Param {
            name,
            ty,
            kind,
            docs,
        })
    }
}

impl Param {
    pub fn docs(&self) -> TokenStream {
        let mut out = quote! {};
        for doc in self.docs.iter() {
            out = quote! { #out #doc }
        }
        out
    }
    pub fn skip(&self) -> Option<&Expr> {
        match self.kind {
            ParamKind::Skip(ref expr) => Some(expr),
            _ => None
        }
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
            let docs = field
                .attrs
                .iter()
                .filter(|a| a.path().is_ident("doc"))
                .cloned()
                .collect();
            let Some(name) = field.ident.clone() else {
                throw!(field, "#[derive({})] only supports named structs. You can use `{}!` for complex cases.", name, alter);
            };
            let kind = field.attrs
                .iter()
                .filter(|a| a.path().is_ident("param"))
                .map(|a| a.parse_args())
                .last()
                .or(Some(Ok(ParamKind::Common)))
                .unwrap()?;
            params.push(Param {
                ty,
                name,
                kind,
                docs,
            });
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
            let docs = param.docs();
            if let Some(skip) = param.skip() {
                param_values = quote! { #param_values #ident: #skip, };
            } else {
                param_values = quote! { #param_values #ident, };
                type_params = quote! { #type_params #mod_ident::#ident, };
                type_params_deconstruct =
                    quote! { #type_params_deconstruct #mod_ident::#ident(mut #ident), };
                fields = quote! { #fields
                    #[allow(unused_variables)]
                    #docs
                    pub #ident: #lib::Param<#ident, #param_ty>,
                };
                fields_new = quote! { #fields_new #ident: #lib::Param(::std::marker::PhantomData), };
                let default = match &param.kind {
                    ParamKind::Default(default) => {
                        quote! {
                            impl Default for #ident {
                                fn default() -> Self {
                                    #ident(#default)
                                }
                            }
                        }
                    }
                    ParamKind::Common => {
                        quote! {
                            impl Default for #ident {
                                fn default() -> Self {
                                    #ident(Default::default())
                                }
                            }
                        }
                    },
                    ParamKind::Required => {
                        quote! {}
                    },
                    ParamKind::Skip(skip) => {
                        throw!(skip, "Unexected skip param");
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
    pub this: Type,
    pub segments: Vec<Type>,
    pub next: Type,
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
            let ty = &input.ident;
            return Ok(Sequence {
                this: parse_quote! { #ty },
                next: parse_quote! { Nothing },
                segments: vec![],
            })
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
        let decls = input.parse::<Declarations>()?;
        let ty = decls.parse_declaration("seg")?;
        let constructor: Constructor = decls.parse_declaration("construct")?;
        let params = constructor.params;
        let body = Some(constructor.expr);
        let props = decls.parse_or_default("props")?;
        Ok(DeriveSegment {
            ty,
            params,
            body,
            props,
        })
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
        Ok(DeriveSegment {
            ty,
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
        let props_descriptors = self.props.build_type_descriptors(ctx, &ty)?;
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

            pub struct Params<T: #lib::Singleton> {
                #fields
                __base__: ::std::marker::PhantomData<T>,
            }
            impl<T: #lib::Singleton> #lib::Singleton for Params<T> {
                fn instance() -> &'static Self {
                    &Params {
                        #fields_new
                        __base__: ::std::marker::PhantomData,
                    }
                }
            }
            impl<T: #lib::Singleton + 'static> std::ops::Deref for Params<T> {
                type Target = T;
                fn deref(&self) -> &Self::Target {
                    T::instance()
                }
            }

            // Props
            pub struct TypeReference;
            impl #lib::TypeReference for TypeReference {
                type Type = #ty;
            }
            pub struct Props<M: 'static, T: #lib::Props<M>>(
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
                #lib::Props<#lib::Set> +
                #lib::Props<#lib::Describe>
            {
                pub fn getters(&self) -> &'static Props<#lib::Get, T> {
                    <Props<#lib::Get, T> as #lib::Singleton>::instance()
                }
                #[doc(hidden)]
                pub fn setters(&self) -> &'static Props<#lib::Set, T> {
                    <Props<#lib::Set, T> as #lib::Singleton>::instance()
                }
                #[doc(hidden)]
                pub fn descriptors(&self) -> &'static Props<#lib::Describe, T> {
                    <Props<#lib::Describe, T> as #lib::Singleton>::instance()
                }
            }
            impl<T: #lib::Props<#lib::Get>> Props<#lib::Get, T> {
                #props_getters
            }
            #[doc(hidden)]
            impl<T: #lib::Props<#lib::Set>> Props<#lib::Set, T> {
                #props_setters
            }
            impl<T: #lib::Props<#lib::Describe>> Props<#lib::Describe, T> {
                #props_descriptors
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
            impl<M: 'static, T: #lib::Props<M>> #lib::Singleton for Props<M, T> {
                fn instance() -> &'static Self {
                    &Props(::std::marker::PhantomData)
                }
            }
            impl<M: 'static, T: #lib::Props<M>> #lib::Props<M> for Props<M, T> { }


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
                type Props<M: 'static, T: #lib::Props<M> + 'static> = #mod_ident::Props<M, T>;
                type Params<T: #lib::Singleton + 'static> = #mod_ident::Params<T>;
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

pub struct Prop {
    pub ident: Ident,
    pub ty: Type,
    pub kind: PropKind,
    docs: Vec<Attribute>,
}

pub enum PropKind {
    Value,
    Construct,
    GetSet(Ident, Ident),
}

impl Parse for Prop {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let docs = Attribute::parse_outer(&input)?
            .iter()
            .filter(|a| a.path().is_ident("doc"))
            .cloned()
            .collect();
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        input.parse::<Token![=]>()?;
        let kind = if let Ok(ident) = input.parse::<Ident>() {
            if &ident.to_string() == "construct" {
                PropKind::Construct
            } else if &ident.to_string() == "value" {
                PropKind::Value
            } else {
                throw!(ident, "Expected construct|value|[get,set].");
            }
        } else {
            let content;
            bracketed!(content in input);
            let get = content.parse()?;
            content.parse::<Token![,]>()?;
            let set = content.parse()?;
            if !content.is_empty() {
                throw!(content, "Unexpected input for prop [get, set].");
            }
            PropKind::GetSet(get, set)
        };
        Ok(Prop {
            docs,
            ident,
            kind,
            ty,
        })
    }
}

impl Prop {
    pub fn from_field(field: &Field) -> syn::Result<Self> {
        let ty = field.ty.clone();
        let Some(ident) = field.ident.clone() else {
            throw!(field, "Anonymous fields not supported.");
        };
        let docs = field
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("doc"))
            .cloned()
            .collect();
        let mut attrs = field.attrs.iter().filter(|a| a.path().is_ident("prop"));
        if let Some(attr) = attrs.next() {
            let spec = attr.parse_args_with(PropSpec::parse)?;
            if spec.construct() {
                Ok(Prop {
                    ident,
                    ty,
                    docs,
                    kind: PropKind::Construct,
                })
            } else {
                let (get, set) = spec.getset()?;
                Ok(Prop {
                    ident,
                    ty,
                    docs,
                    kind: PropKind::GetSet(get, set),
                })
            }
        } else if let Some(ident) = field.ident.clone() {
            Ok(Prop {
                ty,
                ident,
                docs,
                kind: PropKind::Value,
            })
        } else {
            throw!(
                field,
                "#[param(get, set)] is required for unnamed struct fields"
            );
        }
    }
    pub fn docs(&self) -> TokenStream {
        let mut out = quote! {};
        for attr in self.docs.iter() {
            out = quote! { #out #attr }
        }
        out
    }
    pub fn build_getter(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let lib = ctx.constructivism();
        let ident = &self.ident;
        let ty = &self.ty;
        let docs = self.docs();
        Ok(match &self.kind {
            PropKind::Value => quote! {
                #docs
                pub fn #ident(self) -> #lib::Value<'a, #ty> {
                    #lib::Value::Ref(&self.0.#ident)
                }
            },
            PropKind::Construct => quote! {
                #docs
                pub fn #ident(self) -> <#ty as #lib::ConstructItem>::Getters<'a> {
                    <<#ty as #lib::ConstructItem>::Getters<'a> as #lib::Getters<'a, #ty>>::from_ref(
                        &self.0.#ident
                    )
                }
            },
            PropKind::GetSet(get, _set) => quote! {
                #docs
                pub fn #ident(self) -> #lib::Value<'a, #ty> {
                    #lib::Value::Val(self.0.#get())
                }
            },
        })
    }
    pub fn build_setter(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let lib = ctx.path("constructivism");
        let ty = &self.ty;
        let ident = &self.ident;
        Ok(match &self.kind {
            PropKind::Value => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    #[doc(hidden)]
                    pub fn #setter(self, __value__: #ty) {
                        self.0.#ident = __value__;
                    }
                }
            }
            PropKind::Construct => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    #[doc(hidden)]
                    pub fn #ident(self) -> <#ty as #lib::ConstructItem>::Setters<'a> {
                        <<#ty as #lib::ConstructItem>::Setters<'a> as #lib::Setters<'a, #ty>>::from_mut(
                            &mut self.0.#ident
                        )
                    }
                    #[doc(hidden)]
                    pub fn #setter(self, __value__: #ty) {
                        self.0.#ident = __value__;
                    }
                }
            }
            PropKind::GetSet(_get, set) => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    #[doc(hidden)]
                    pub fn #ident(self, __value__: #ty) {
                        self.0.#set(__value__);
                    }
                    #[doc(hidden)]
                    pub fn #setter(self, __value__: #ty) {
                        self.0.#set(__value__);
                    }
                }
            }
        })
    }

    pub fn build_lookup_getter(&self, ctx: &Context, this: &Type) -> syn::Result<TokenStream> {
        let lib = ctx.constructivism();
        let ident = &self.ident;
        let ty = &self.ty;
        let docs = self.docs();
        Ok(match &self.kind {
            PropKind::Value => {
                quote! {
                    #docs
                    pub fn #ident<'a>(&self, __this__: &'a #this) -> #lib::Value<'a, #ty> {
                        #lib::Value::Ref(&__this__.#ident)
                    }
                }
            }
            PropKind::Construct => {
                quote! {
                    #docs
                    pub fn #ident<'a>(&self, __this__: &'a #this) -> <#ty as #lib::ConstructItem>::Getters<'a> {
                        <<#ty as #lib::ConstructItem>::Getters<'a> as #lib::Getters<'a, #ty>>::from_ref(
                            &__this__.#ident
                        )
                    }
                }
            }
            PropKind::GetSet(get, _set) => {
                quote! {
                    #docs
                    pub fn #ident<'a>(&self, __this__: &'a #this) -> #lib::Value<'a, #ty> {
                        #lib::Value::Val(__this__.#get())
                    }
                }
            }
        })
    }

    pub fn build_lookup_setter(&self, ctx: &Context, this: &Type) -> syn::Result<TokenStream> {
        let lib = ctx.constructivism();
        let ident = &self.ident;
        let ty = &self.ty;
        Ok(match &self.kind {
            PropKind::Value => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    #[doc(hidden)]
                    pub fn #ident(&self, __this__: &mut #this, __value__: #ty) {
                        __this__.#ident = __value__;
                    }
                    #[doc(hidden)]
                    pub fn #setter(&self, __this__: &mut #this, __value__: #ty) {
                        __this__.#ident = __value__;
                    }
                }
            }
            PropKind::Construct => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    #[doc(hidden)]
                    pub fn #ident<'a>(&self, __this__: &'a mut #this) -> <#ty as #lib::ConstructItem>::Setters<'a> {
                        <<#ty as #lib::ConstructItem>::Setters<'a> as #lib::Setters<'a, #ty>>::from_mut(
                            &mut __this__.#ident
                        )
                    }
                    #[doc(hidden)]
                    pub fn #setter(&self, __this__: &mut #this, __value__: #ty) {
                        __this__.#ident = __value__;
                    }
                }
            }
            PropKind::GetSet(_get, set) => {
                let setter = format_ident!("set_{}", ident);
                quote! {
                    #[doc(hidden)]
                    pub fn #ident(&self, __this__: &mut #this, __value__: #ty) {
                        __this__.#set(__value__);
                    }
                    #[doc(hidden)]
                    pub fn #setter(&self, __this__: &mut #this, __value__: #ty) {
                        __this__.#set(__value__);
                    }
                }
            }
        })
    }

    pub fn build_type_descriptor(&self, _ctx: &Context, _this: &Type) -> syn::Result<TokenStream> {
        let ident = &self.ident;
        Ok(quote! {
            pub fn #ident(&self) -> &'static TypeReference {
                &TypeReference
            }
        })
    }
}

pub struct PropSpec(pub Vec<Ident>);

impl Parse for PropSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PropSpec(
            input
                .parse_terminated(Ident::parse, Token![,])?
                .into_iter()
                .collect(),
        ))
    }
}
impl PropSpec {
    pub fn skip(&self) -> bool {
        self.0.len() == 1 && &self.0[0].to_string() == "skip"
    }
    pub fn construct(&self) -> bool {
        self.0.len() == 1 && &self.0[0].to_string() == "construct"
    }
    pub fn getset(&self) -> syn::Result<(Ident, Ident)> {
        if self.0.len() == 2 {
            Ok((self.0[0].clone(), self.0[1].clone()))
        } else {
            throw!(self.0[0], "Expected #[prop(getter, setter)]");
        }
    }
}

#[derive(Default)]
pub struct Props(Vec<Prop>);
impl Parse for Props {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        Ok(Props(
            content
                .parse_terminated(Prop::parse, Token![;])?
                .into_iter()
                .collect(),
        ))
    }
}
impl Props {
    pub fn from_fields(fields: &Fields) -> syn::Result<Self> {
        let mut props = vec![];
        for field in fields.iter() {
            if let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("prop")) {
                // attr.meta.require_list()?.
                // attr.meta.to_token_stream()
                // Punctuated::<Ident, Token![,]>::parse_terminated(attr.meta.to_token_stream())?;
                // attr.parse_args::<Punctuated<Ident, Token![,]>>()?;
                let skip = attr.parse_args_with(PropSpec::parse)?.skip();
                if skip {
                    continue;
                }
            }
            props.push(Prop::from_field(field)?)
        }
        Ok(Props(props))
    }
    pub fn build_getters(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let mut out = quote! {};
        for prop in self.iter() {
            let getter = prop.build_getter(ctx)?;
            out = quote! { #out #getter }
        }
        Ok(out)
    }
    pub fn build_setters(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let mut out = quote! {};
        for prop in self.iter() {
            let setter = prop.build_setter(ctx)?;
            out = quote! { #out #setter }
        }
        Ok(out)
    }

    pub fn build_lookup_getters(&self, ctx: &Context, this: &Type) -> syn::Result<TokenStream> {
        let mut out = quote! {};
        for prop in self.iter() {
            let getter = prop.build_lookup_getter(ctx, this)?;
            out = quote! { #out #getter }
        }
        Ok(out)
    }

    pub fn build_lookup_setters(&self, ctx: &Context, this: &Type) -> syn::Result<TokenStream> {
        let mut out = quote! {};
        for prop in self.iter() {
            let setter = prop.build_lookup_setter(ctx, this)?;
            out = quote! { #out #setter }
        }
        Ok(out)
    }

    pub fn build_type_descriptors(&self, ctx: &Context, this: &Type) -> syn::Result<TokenStream> {
        let mut out = quote! {};
        for prop in self.iter() {
            let descriptor = prop.build_type_descriptor(ctx, this)?;
            out = quote! { #out #descriptor };
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

pub struct Constructor {
    params: Vec<Param>,
    expr: Expr,
}

impl Parse for Constructor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let params = content.parse_terminated(Param::parse, Token![,])?;
        let params = params.into_iter().collect();
        input.parse::<Token![->]>()?;
        let expr = input.parse()?;
        Ok(Constructor { params, expr })
    }
}

pub struct DeriveConstruct {
    pub ty: Type,
    pub sequence: Sequence,
    pub params: Vec<Param>,
    pub props: Props,
    pub body: Option<Expr>,
}

impl Parse for DeriveConstruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let decls = input.parse::<Declarations>()?;
        let sequence: Sequence = decls.parse_declaration("seq")?;
        let ty = sequence.this.clone();
        let constructor: Constructor = decls.parse_declaration("construct")?;
        let params = constructor.params;
        let body = Some(constructor.expr);
        let props = decls.parse_or_default("props")?;
        Ok(DeriveConstruct {
            ty,
            params,
            body,
            sequence,
            props,
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

    pub fn mod_ident(&self) -> syn::Result<Ident> {
        let type_ident = self.ty.as_ident()?;
        Ok(format_ident!(
            "{}_construct",
            type_ident.to_string().to_lowercase()
        ))
    }

    pub fn design_ident(&self) -> syn::Result<Ident> {
        let type_ident = self.ty.as_ident()?;
        Ok(format_ident!(
            "{}Design",
            type_ident.to_string()
        ))
    }

    pub fn build(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let ty = &self.ty;
        let lib = ctx.path("constructivism");
        let type_ident = ty.as_ident()?;
        let mod_ident = self.mod_ident()?;
        let design = self.design_ident()?;
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
        let props_descriptors = self.props.build_type_descriptors(ctx, &ty)?;
        let getters = self.props.build_getters(ctx)?;
        let setters = self.props.build_setters(ctx)?;
        let decls = {
            let base = &self.sequence.next;
            let base = if !base.is_nothing() {
                quote! { #base }
            } else {
                quote! { () }
            };
            let mut deref_fields = quote! { <#base as #lib::Construct>::Params };
            let mut deref_props = quote! { <#base as #lib::Construct>::Props<M> };
            deref_design = quote! { <#base as #lib::Construct>::Design };
            for segment in self.sequence.segments.iter() {
                deref_fields = quote! { <#segment as #lib::Segment>::Params<#deref_fields> };
                deref_design = quote! { <#segment as #lib::Segment>::Design<#deref_design> };
                deref_props = quote! { <#segment as #lib::Segment>::Props<M, #deref_props> };
            }

            quote! {
                // Params
                pub struct Params {
                    #fields
                }
                impl #lib::Singleton for Params {
                    fn instance() -> &'static Self {
                        &Params {
                            #fields_new
                        }
                    }
                }
                impl ::std::ops::Deref for Params {
                    type Target = #deref_fields;
                    fn deref(&self) -> &Self::Target {
                        <#deref_fields as #lib::Singleton>::instance()
                    }
                }

                // Props
                pub struct Props<M: 'static>(::std::marker::PhantomData<M>);
                pub struct Getters<'a>(&'a #ty);
                pub struct Setters<'a>(&'a mut #ty);
                pub struct TypeReference;
                impl #lib::TypeReference for TypeReference {
                    type Type = #ty;
                }
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
                    #[doc(hidden)]
                    pub fn setters(&self) -> &'static Props<#lib::Set> {
                        <Props<#lib::Set> as #lib::Singleton>::instance()
                    }
                    #[doc(hidden)]
                    pub fn descriptors(&self) -> &'static Props<#lib::Describe> {
                        <Props<#lib::Describe> as #lib::Singleton>::instance()
                    }
                }
                impl Props<#lib::Describe> {
                    #props_descriptors
                }
                impl Props<#lib::Get> {
                    #props_getters
                }
                #[doc(hidden)]
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
                impl<M: 'static> #lib::Singleton for Props<M> {
                    fn instance() -> &'static Self {
                        &Props(::std::marker::PhantomData)
                    }
                }
                impl<M: 'static> #lib::Props<M> for Props<M> { }
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
                    type Params = #mod_ident::Params;
                    type Props<M: 'static> = #mod_ident::Props<M>;
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
