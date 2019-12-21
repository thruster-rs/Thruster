extern crate proc_macro;
extern crate proc_macro2;
extern crate lazy_static;
extern crate uuid;

use crate::proc_macro::{TokenStream};
use crate::proc_macro2::{Ident, Span as Span2};
use quote::quote;
use syn;

#[proc_macro_attribute]
pub fn middleware_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    if let syn::Item::Fn(mut function_item) = syn::parse(item.clone()).unwrap() {
        let name = function_item.ident.clone();
        let new_name = Ident::new(&format!("__async_{}", name.clone()), Span2::call_site());
        function_item.ident = new_name.clone();

        let visibility = function_item.vis.clone();
        let arguments = function_item.decl.inputs.clone();

        let context_type = match &arguments[0] {
            syn::FnArg::Captured(cap) => &cap.ty,
            _ => panic!("Expected the first argument to be a context type")
        };

        let gen = quote! {
            #function_item

            #visibility fn #name(ctx: #context_type, next: MiddlewareNext<#context_type>) -> MiddlewareReturnValue<#context_type> {
                Box::pin(#new_name(ctx, next))
            }
        };

        // proc_macro::Span::call_site()
        //     .note("Thruster code output")
        //     .note(gen.to_string())
        //     .emit();

        gen.into()
    } else {
        item
    }
}
