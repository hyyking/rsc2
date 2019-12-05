extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

mod tryinto;
mod wrapenum;

/// Builds a coordinator on top of an [`Iterator`](std::iter::Iterator) of requests
#[cfg(feature = "run")]
#[proc_macro_attribute]
pub fn run(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let _ = syn::parse_macro_input!(args as syn::parse::Nothing);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    if !input.sig.inputs.is_empty() {
        let msg = "this function cannot accept arguments";
        return syn::Error::new_spanned(&input.sig.inputs, msg)
            .to_compile_error()
            .into();
    }

    let result = quote! {
        #(#attrs)*
        fn #name() #ret {
            let coordinator = rsc2::runtime::Coordinator::new();
            coordinator.run(#body)
        }
    };

    result.into()
}

/// Implements [`Into`](std::convert::Into) for types that are wrapped in an enum
#[cfg(feature = "derive")]
#[proc_macro_derive(WrapEnum)]
pub fn macro_wrap_enum(t: TokenStream) -> TokenStream {
    wrapenum::try_wrap_enum(t).unwrap()
}

/// Implements [`TryFrom`](std::conver::TryFrom) for C-like enums
#[cfg(feature = "derive")]
#[proc_macro_derive(TryIntoEnum)]
pub fn macro_try_into_enum(t: TokenStream) -> TokenStream {
    tryinto::try_into_enum(t).unwrap()
}
