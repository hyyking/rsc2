use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DataEnum, DeriveInput, Error};

pub(super) fn try_into_enum(t: TokenStream) -> Result<TokenStream, Error> {
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
