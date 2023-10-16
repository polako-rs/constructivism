use proc_macro2::{Ident, TokenStream};
use syn::{braced, parenthesized, parse::Parse, spanned::Spanned, Expr, Token, Type, token::Brace};

use quote::{format_ident, quote};

use crate::{context::Context, throw};


pub trait ContextLike {
    fn path(&self, name: &'static str) -> TokenStream;
    fn constructivism(&self) -> TokenStream {
        self.path("constructivism")
    }
}

impl ContextLike for Context {
    fn path(&self, name: &'static str) -> TokenStream {
        self.path(name)
    }
}


pub trait Value: Parse + Clone {
    type Context: ContextLike;
    fn build(item: &Self, ctx: &Self::Context) -> syn::Result<TokenStream>;
    fn parse2(stream: TokenStream) -> syn::Result<Self> {
        syn::parse2::<Self>(stream)
    }
}

impl Value for Expr {
    type Context = Context;
    fn build(item: &Self, _: &Self::Context) -> syn::Result<TokenStream> {
        Ok(quote! { #item })
    }
}

#[derive(Clone)]
pub struct Param<V: Value> {
    pub ident: Ident,
    pub value: V,
}

impl<V: Value> Parse for Param<V> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut value = None;
        // this is kinda autocomplete
        let dot = input.parse::<Token![.]>()?;
        if input.is_empty() || input.peek(Token![,]) {
            let ident = format_ident!("DOT_AUTOCOMPLETE_TOKEN", span = dot.span());
            let value = V::parse2(quote! { true })?;
            return Ok(Param { ident, value });
        }
        let ident: Ident = input.parse()?;
        if value.is_none() && input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            value = Some(V::parse(input)?);
        }
        if value.is_none() && (input.is_empty() || input.peek(Token![,])) {
            value = Some(syn::parse_quote_spanned! { ident.span() =>
                true
            });
        }
        if value.is_none() {
            throw!(input, "Unexpected param input: {}", input.to_string());
        }
        Ok(Param {
            ident,
            value: value.unwrap(),
        })
    }
}
impl<V: Value> Param<V> {
    //         let param: &$crate::Param<_, _> = &$fields.$f;
    //         let field = param.field();
    //         let value = $params.field(&field).define(param.value($e.into()));
    //         let $params = $params + value;
    pub fn build(&self, ctx: &V::Context) -> syn::Result<TokenStream> {
        let ident = &self.ident;
        let value = V::build(&self.value, ctx)?;
        let lib = ctx.path("constructivism");
        Ok(quote! {
            let param: &#lib::Param<_, _> = &fields.#ident;
            let field = param.field();
            let value = params.field(&field).define(param.value((#value).into()));
            let params = params + value;
        })
    }
}

#[derive(Clone)]
pub struct Params<V: Value> {
    pub items: Vec<Param<V>>,
}

impl<V: Value> Parse for Params<V> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = vec![];
        while !input.is_empty() {
            items.push(Param::<V>::parse(input)?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Params { items })
    }
}

impl<V: Value> Params<V> {
    pub fn new() -> Self {
        Params { items: vec![] }
    }
    pub fn build(&self, ctx: &V::Context) -> syn::Result<TokenStream> {
        let mut out = quote! {};
        for param in self.items.iter() {
            let param = param.build(ctx)?;
            out = quote! { #out #param }
        }
        Ok(out)
    }
    pub fn empty() -> Self {
        Params { items: vec![] }
    }
    pub fn parenthesized(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        content.parse()
    }

    pub fn braced(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        content.parse()
    }
}
pub struct Construct<V: Value> {
    pub ty: Type,
    pub flattern: bool,
    pub params: Params<V>,
}

impl<V: Value> Parse for Construct<V> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;
        let mut flattern = true;
        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            flattern = false;
        }
        let params = if input.peek(Brace) {
            Params::braced(input)?
        } else {
            Params::new()
        };
        Ok(Construct {
            ty,
            flattern,
            params,
        })
    }
}

// #[macro_export]
// macro_rules! construct {
//     ($t:ty { $($rest:tt)* } ) => {
//         {
//             use $crate::traits::*;
//             type Params = <$t as $crate::Construct>::Params;
//             let fields = <<$t as $crate::Construct>::Params as $crate::Singleton>::instance();
//             let params = <<$t as $crate::Construct>::ExpandedParams as $crate::Extractable>::as_params();
//
//             // body here, see Param::build(..)
//
//             let defined_params = params.defined();
//             <$t as $crate::Construct>::construct(defined_params).flattern()
//         }
//     };
// }
impl<V: Value> Construct<V> {
    pub fn build(&self, ctx: &V::Context) -> syn::Result<TokenStream> {
        let lib = ctx.path("constructivism");
        let ty = &self.ty;
        let flattern = if self.flattern {
            quote! { .flattern() }
        } else {
            quote! {}
        };
        let body = self.params.build(ctx)?;
        Ok(quote! {{
            use #lib::traits::*;
            let fields = <<#ty as #lib::Construct>::Params as #lib::Singleton>::instance();
            let params = <<#ty as #lib::Construct>::ExpandedParams as #lib::Extractable>::as_params();
            #body
            let defined_params = params.defined();
            <#ty as #lib::Construct>::construct(defined_params)#flattern
        }})
    }
}

pub struct Prop {
    pub root: Type,
    pub path: Vec<Ident>,
}

impl Parse for Prop {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let root = input.parse()?;
        let mut path = vec![];
        while input.peek(Token![.]) {
            let dot = input.parse::<Token![.]>()?;
            if input.is_empty() {
                path.push(format_ident!("DOT_AUTOCOMPLETE_TOKEN", span = dot.span()))
            } else {
                path.push(input.parse()?)
            }
        }
        Ok(Prop { root, path })
    }
}

impl Prop {
    pub fn build(&self, ctx: &Context) -> syn::Result<TokenStream> {
        let lib = ctx.constructivism();
        let root = &self.root;
        let mut get = quote! { <<#root as #lib::Construct>::Props<#lib::Lookup> as #lib::Singleton>::instance().getters() };
        let mut set = quote! { <<#root as #lib::Construct>::Props<#lib::Lookup> as #lib::Singleton>::instance().setters() };
        if self.path.len() == 0 {
            throw!(self.root, "Missing property path.");
        }
        let last = self.path.len() - 1;

        for (idx, part) in self.path.iter().enumerate() {
            let setter = format_ident!("set_{}", part);
            if idx == 0 {
                get = quote! { #get.#part(host) };
            } else {
                get = quote! { #get.#part() };
            }

            if idx == 0 && idx == last {
                set = quote! { #set.#setter(host, value) };
            } else if idx == last {
                set = quote! { #set.#setter(value)};
            } else if idx == 0 {
                set = quote! { #set.#part(host) };
            } else {
                set = quote! { #set.#part() };
            }
        }
        get = quote! { #get.into_value() };
        Ok(quote! {
            #lib::Prop::new(
                |host| #get,
                |host, value| #set
            )
        })
    }
}
