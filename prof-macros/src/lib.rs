use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn prof(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);
    let sig = &func.sig;
    let name = &func.sig.ident.to_string();
    let body = &func.block;
    let wrap = quote! {
        #sig {
            let prof_anchor = prof::AnchorPoint::new(#name);
            #body
        }
    };

    TokenStream::from(wrap)
}
