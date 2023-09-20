use crate::context::Context;
use crate::exts::TypeExt;
use crate::throw;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    bracketed, parenthesized, parse::Parse, spanned::Spanned, Data, DeriveInput, Expr, Ident,
    Token, Type,
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
    body: Option<Expr>,
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
        Ok(DeriveSegment { ty, params, body })
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
        let construct = if let Some(expr) = &self.body {
            expr.clone()
        } else {
            syn::parse2(quote! {
                Self { #param_values }
            })
            .unwrap()
        };
        let decls = quote! {
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
        };
        Ok(quote! {
            mod #mod_ident {
                use super::*;
                #decls
                #impls
            }
            impl #lib::ConstructItem for #type_ident {
                type Params = ( #type_params );
                fn construct_item(params: Self::Params) -> Self {
                    let (#type_params_deconstruct) = params;
                    #construct
                }
            }
            impl #lib::Segment for #type_ident {
                type Fields<T: #lib::Singleton + 'static> = #mod_ident::Fields<T>;
                type Design<T: #lib::Singleton + 'static> = #design<T>;
            }
            pub struct #design<T: #lib::Singleton>(
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

pub struct DeriveConstruct {
    ty: Type,
    sequence: Sequence,
    params: Vec<Param>,
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
        let body = None;
        Ok(DeriveConstruct {
            ty,
            sequence,
            params,
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
        let decls = {
            let base = &self.sequence.next;
            let base = if !base.is_nothing() {
                quote! { #base }
            } else {
                quote! { () }
            };
            let mut deref_fields = quote! { <#base as #lib::Construct>::Fields };
            deref_design = quote! { <#base as #lib::Construct>::Design };
            for segment in self.sequence.segments.iter() {
                deref_fields = quote! { <#segment as #lib::Segment>::Fields<#deref_fields> };
                deref_design = quote! { <#segment as #lib::Segment>::Design<#deref_design> };
            }

            quote! {
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
