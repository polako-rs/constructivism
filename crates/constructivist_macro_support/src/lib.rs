use syn::{parse::Parse, parse_macro_input};
use quote::*;
use proc_macro;
use proc_macro2::TokenStream;



#[proc_macro]
pub fn construct_implementations(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let construct = parse_macro_input!(input as ConstructImplementations);
    proc_macro::TokenStream::from(construct.construct())
}

struct ConstructImplementations {
    size: u16
}

impl Parse for ConstructImplementations {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let size: syn::Lit = input.parse()?;
        let span = size.span();
        let syn::Lit::Int(size) = size else {
            return Err(syn::Error::new(span, "Expected int literal."))
        };
        let Ok(size) = size.base10_parse::<u16>() else {
            return Err(syn::Error::new(span, "Expected int literal."))
        };
        Ok(ConstructImplementations { size })
    }
}

impl ConstructImplementations {
    pub fn construct(&self) -> TokenStream {

        let mut structs = quote! { };
        let mut impls = quote! { };
        for i in 0..self.size {
            let di = format_ident!("D{:02}", i);
            let ui = format_ident!("U{:02}", i);
            let fi = format_ident!("F{:02}", i);
            structs = quote! { #structs
                struct #di<T>(T);
                struct #ui<T>(PhantomData<T>);
                struct #fi<T>(PhantomData<T>);
                impl<T> #fi<T> {
                    fn define(self, value: T) -> #di<T> {
                        #di(value)
                    }
                }
            };
        }
        for c in 0..self.size {
            for i in 0..c {
                impls = quote! { #impls impl < };
                for j in 0..i {
                    let ai = format_ident!("A{:02}", j);
                    if i == j {
                        let ti = format_ident!("T{:02}", i);
                        impls = quote! { #impls #ti, };
                    } else {
                        impls = quote! { #impls #ai, };
                    }
                }
                impls = quote!(#impls > ExtractField< );
                for j in 0..i {
                    let tj = format_ident!("T{:02}", j);
                    if i == j {
                        let fi = format_ident!("F{:02}", i);
                        impls = quote!(#impls #fi<#tj>,);
                    } else {
                        impls = quote!(#impls #tj,);
                    }
                }

                let mut pargs = quote! { };
                // impls = quote!{ #impls > for Params<(#impls)> };
                let ui = format_ident!("U{:02}", i);
                let ti = format_ident!("T{:02}", i);
                let fi = format_ident!("F{:02}", i);
                for j in 0..i {
                    let aj = format_ident!("A{:02}", j);
                    if i == j {
                        pargs = quote!(#pargs #ui<#ti>,);
                    } else {
                        pargs = quote!(#pargs #aj,);
                    }
                }
                impls = quote! { #impls > for Params<(#pargs)> {
                    fn extract_field(&self, _: &Field<#ti>) -> #fi<#ti> {
                        #fi(PhantomData)
                    }
                }}
            }
        }
        // for j in 0..i {
        //     impl<T0, A1> ExtractField<F0<T0>, T0> for Params<(U0<T0>, A1)> {
        //         fn extract_field(&self, _: &Field<T0>) -> F0<T0> {
        //             F0(PhantomData)
        //         }
        //     }
        // }
        quote! { 
            use std::marker::PhantomData;
            
            #structs

            // #impls
        }
    }
}




