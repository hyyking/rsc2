extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;

use syn::{Data, DataEnum, DeriveInput, Error, Fields, FieldsUnnamed, Variant};

fn try_wrap_enum(t: TokenStream) -> Result<TokenStream, Error> {
    let input: DeriveInput = syn::parse(t)?;

    let ident = input.ident;

    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("only Enums"),
    };

    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        panic!("No generics allowed")
    }
    let types = variants.iter().map(
        |Variant {
             ident: variant_ident,
             fields: variant_fields,
             ..
         }| {
            match variant_fields {
                Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => match unnamed.len() {
                    1 => {
                        if let syn::Type::Path(e) = unnamed.first().unwrap().ty.clone() {
                            return quote! {
                                impl Into<#ident> for #e {
                                    fn into(self) -> #ident {
                                        #ident::#variant_ident(self)
                                    }
                                }
                            };
                        } else {
                            panic!()
                        }
                    }
                    _ => panic!(),
                },
                _ => panic!(),
            };
        },
    );
    let impls = quote! {#(#types)*};
    Ok(impls.into())
}

#[proc_macro_derive(WrapEnum)]
pub fn wrap_enum(t: TokenStream) -> TokenStream {
    try_wrap_enum(t).unwrap()
}

/*
       let c = data.variants.iter().map(|variant| {
           let iter = if let syn::Fields::Unnamed(v) = variant.fields {
               let z = v.unnamed.iter().map(|f| {
                   if let syn::Type::Path(p) = f.ty {
                       println!("{:#?}", p);
                       quote! {/* ... */}
                   } else {
                       quote! {/* ... */}
                   }
               });
               quote! {$(#z)*}
           } else {
               let err = Error::new(variant.fields.clone().span(), "Expected unnamed field")
                   .to_compile_error();
               quote! {#err}
           };
           quote! {$(#iter)*}
       });
       quote! {$(#c)*}
*/
