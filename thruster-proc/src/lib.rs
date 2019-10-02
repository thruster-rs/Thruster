#![feature(proc_macro_diagnostic, proc_macro_span)]
#![feature(proc_macro_hygiene)]

extern crate proc_macro;
extern crate proc_macro2;
extern crate lazy_static;
extern crate uuid;

use crate::proc_macro::{TokenStream};
use crate::proc_macro2::TokenTree;
use crate::proc_macro2::{TokenStream as TokenStream2};
use crate::proc_macro2::{Ident, Span as Span2};
use crate::uuid::Uuid;
use quote::quote;
use syn;

#[proc_macro]
pub fn async_middleware(item: TokenStream) -> TokenStream {
    let mut stream_iter = TokenStream2::from(item).into_iter();

    let context_type = stream_iter.next().unwrap();
    let _ = stream_iter.next();
    let fn_array = stream_iter.next().unwrap();

    let fn_group = match fn_array.clone() {
        TokenTree::Group(group) => group,
        _ => panic!("Uh oh!")
    };

    let unique_id = format!("ident_{}", Uuid::new_v4()).replace("-", "");
    let class_ident = Ident::new(&format!("Middleware_{}", unique_id), Span2::call_site());
    let instance_ident = Ident::new(&format!("middleware_{}", unique_id), Span2::call_site());

    let gen = quote! {{
        use thruster::{Chain, Middleware, MiddlewareChain, MiddlewareNext, MiddlewareReturnValue};

        const #class_ident: &'static [
            fn(#context_type, MiddlewareNext<#context_type>) -> MiddlewareReturnValue<#context_type>
        ] = &#fn_group;

        static #instance_ident: Middleware<#context_type> = Middleware {
            middleware: #class_ident
        };

        let chain = Chain::new(vec![&#instance_ident]);

        MiddlewareChain {
            chain,
            assigned: true
        }
    }};

    gen.into()
}

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
