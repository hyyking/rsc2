extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;

use syn::{spanned::Spanned, Data, DataEnum, DeriveInput, Error, Fields, FieldsUnnamed};

fn try_wrap_enum(t: TokenStream) -> Result<TokenStream, Error> {
    let input: DeriveInput = syn::parse(t)?;

    let enum_ident = input.ident;
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => {
            let err = Error::new(enum_ident.span(), "Expected an enum").to_compile_error();
            return Ok(quote! {#err}.into());
        }
    };

    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        let err = Error::new(input.generics.span(), "Expected concrete types").to_compile_error();
        return Ok(quote! {#err}.into());
    }
    let into_impls = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        match &variant.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => match unnamed.len() {
                0 => {
                    let err = Error::new(unnamed.span(), "Expected one field in the enum variant")
                        .to_compile_error();
                    quote! {#err}
                }
                1 => {
                    if let syn::Type::Path(inner_type) = unnamed.first().unwrap().ty.clone() {
                        return quote! {
                            impl Into<#enum_ident> for #inner_type {
                                fn into(self) -> #enum_ident {
                                    #enum_ident::#variant_ident(self)
                                }
                            }
                        };
                    } else {
                        let err = Error::new(unnamed.span(), "Expected a path").to_compile_error();
                        quote! {#err}
                    }
                }
                _ => {
                    let err = Error::new(
                        unnamed.span(),
                        "Expected a single field in the enum variant",
                    )
                    .to_compile_error();
                    quote! {#err}
                }
            },
            _ => {
                let err =
                    Error::new(variant.span(), "Expected an unnamed field").to_compile_error();
                quote! {#err}
            }
        }
    });
    let impls = quote! {#(#into_impls)*};
    Ok(impls.into())
}

#[proc_macro_derive(WrapEnum)]
pub fn macro_wrap_enum(t: TokenStream) -> TokenStream {
    try_wrap_enum(t).unwrap()
}

fn try_into_enum(t: TokenStream) -> Result<TokenStream, Error> {
    let input: DeriveInput = syn::parse(t)?;

    let enum_ident = input.ident;
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => {
            let err = Error::new(enum_ident.span(), "Expected an enum").to_compile_error();
            return Ok(quote! {#err}.into());
        }
    };

    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        let err = Error::new(input.generics.span(), "Expected concrete types").to_compile_error();
        return Ok(quote! {#err}.into());
    }
    let into_variants = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        quote! {v if v == (#enum_ident::#variant_ident as i32) => Ok(#enum_ident::#variant_ident)}
    });
    let try_into = quote! {
        impl ::std::convert::TryFrom<i32> for #enum_ident {
            type Error = ();
            fn try_from(value: i32) -> ::std::result::Result<Self, Self::Error> {
                match value {
                    #(#into_variants,)*
                    _ => Err(())
                }

            }
        }
    };
    Ok(try_into.into())
}

#[proc_macro_derive(TryIntoEnum)]
pub fn macro_try_into_enum(t: TokenStream) -> TokenStream {
    try_into_enum(t).unwrap()
}
