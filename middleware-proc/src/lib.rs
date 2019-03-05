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
        let context_name = match &arguments[0] {
            syn::FnArg::Captured(cap) => &cap.pat,
            _ => panic!("Expected the first argument to be a context type")
        };
        let context_type = match &arguments[0] {
            syn::FnArg::Captured(cap) => &cap.ty,
            _ => panic!("Expected the first argument to be a context type")
        };
        let next_name = match &arguments[1] {
            syn::FnArg::Captured(cap) => &cap.pat,
            _ => panic!("Expected the second argument to be next")
        };

        let gen = quote! {
            fn #name(#context_name: #context_type, #next_name: impl Fn(#context_type) -> MiddlewareReturnValue<#context_type> + Send + Sync) -> MiddlewareReturnValue<#context_type> {
                let __old_next = #next_name;
                let #next_name = |ctx| {
                    __old_next(ctx)
                };

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
