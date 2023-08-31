use syn::{parse_macro_input, DeriveInput, Data, Type, TypePath, Expr};
use quote::*;
use proc_macro;
use proc_macro2::TokenStream;



// #[proc_macro]
// pub fn construct_implementations(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let construct = parse_macro_input!(input as ConstructImplementations);
//     proc_macro::TokenStream::from(construct.construct())
// }

// struct ConstructImplementations {
//     size: u16
// }

// impl Parse for ConstructImplementations {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         let size: syn::Lit = input.parse()?;
//         let span = size.span();
//         let syn::Lit::Int(size) = size else {
//             return Err(syn::Error::new(span, "Expected int literal."))
//         };
//         let Ok(size) = size.base10_parse::<u16>() else {
//             return Err(syn::Error::new(span, "Expected int literal."))
//         };
//         Ok(ConstructImplementations { size })
//     }
// }

// impl ConstructImplementations {
//     fn construct(&self) -> TokenStream {
//         quote!()
//     }
// }


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
    let mut type_params = quote! { };                   // slider_construct::min, slider_construct::max, slider_construct::val,
    let mut type_params_deconstruct = quote! { };       // slider_construct::min(min), slider_construct::max(max), slider_construct::val(val),
    let mut param_values = quote! { };                  // min, max, val,
    let mut impls = quote! { };

    for field in input.fields.iter() {
        let Some(ident) = &field.ident else {
            return quote!(compile_error!("#[derive(Construct)] only supports named structs. You can use `constructable!` for complex cases."))
        };
        let field_ty = &field.ty;
        type_params = quote! { #type_params #mod_ident::#ident, };
        type_params_deconstruct = quote! { #type_params_deconstruct #mod_ident::#ident(#ident), };
        param_values = quote! { #param_values #ident, };

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
            impl Fields {
                #[allow(unused)]
                pub fn #ident(&self) -> #lib::Param<#ident, #field_ty> {
                    #lib::Param::new()
                }
            }
        }
    }
    quote! {
        mod #mod_ident {
            use super::*;
            pub struct Fields(::std::marker::PhantomData<<<super::#type_ident as #lib::Construct>::Wraps as #lib::Construct>::Fields>);
            impl #lib::Singleton for Fields {
                fn instance() -> &'static Self {
                    &Fields(::std::marker::PhantomData)
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
            type Params = ( #type_params );
            type Wraps = #wraps;
            type Wrapped = (Self, <Self::Wraps as #lib::Construct>::Wrapped);
            type WrappedParams = (#type_params <Self::Wraps as #lib::Construct>::WrappedParams);
            fn construct_fields() -> &'static Self::Fields {
                <#mod_ident::Fields as #lib::Singleton>::instance()
            }
            fn construct(params: Self::Params) -> Self {
                let (#type_params_deconstruct) = params;
                Self { #param_values }
            }
            fn construct_all<P>(params: P) -> <Self as #lib::Construct>::Wrapped
            where Self: Sized, P: #lib::DefinedValues<
                Self::Params,
                Output = <<<Self as #lib::Construct>::Wraps as #lib::Construct>::WrappedParams as #lib::AsParams>::Defined 
            > {
                let (args, params) = params.extract_values();
                (Self::construct(args), <<Self as #lib::Construct>::Wraps as #lib::Construct>::construct_all(params))
            }
        }
    }
}



trait SupportsInto {
    fn supports_into(&self) -> bool;
}

impl SupportsInto for Type {
    fn supports_into(&self) -> bool {
        match self {
            Type::Path(TypePath { path, .. })  if path.segments.len() == 1 => match &path.segments[0].ident {
                i if i == &format_ident!("usize") => false,
                i if i == &format_ident!("u128") => false,
                i if i == &format_ident!("u64") => false,
                i if i == &format_ident!("u32") => false,
                i if i == &format_ident!("u16") => false,
                i if i == &format_ident!("u8") => false,
                i if i == &format_ident!("isize") => false,
                i if i == &format_ident!("i128") => false,
                i if i == &format_ident!("i64") => false,
                i if i == &format_ident!("i32") => false,
                i if i == &format_ident!("i16") => false,
                i if i == &format_ident!("i8") => false,
                _ => true,
            },
            _ => true,
        }
    }
}

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