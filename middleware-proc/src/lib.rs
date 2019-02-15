extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_attribute]
pub fn async_middleware(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let syn::Item::Fn(function_item) = syn::parse(item.clone()).unwrap() {
        let name = function_item.ident;
        let block = function_item.block;
        let arguments = function_item.decl.inputs;
        let return_type = function_item.decl.output;
        let context_type = match &arguments[0] {
            syn::FnArg::Captured(cap) => &cap.ty,
            _ => panic!("Expected the first argument to be a context type")
        };

        let gen = quote! {
            fn #name(#arguments) -> MiddlewareReturnValue<#context_type> {
                let results = async #block;

                use tokio_async_await::compat::backward;
                Box::new(backward::Compat::new(results))
            }
        };
        gen.into()
    } else {
        item
    }
}
