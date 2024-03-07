// #![feature(proc_macro_diagnostic)]
// extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span as Span2, TokenTree as TokenTree2};
use quote::quote;

#[proc_macro]
pub fn chainable_functions(items: TokenStream) -> TokenStream {
    let items = proc_macro2::TokenStream::from(items);

    let mut idents: Vec<Ident> = items
        .into_iter()
        .filter_map(|v| {
            match v {
                TokenTree2::Ident(ident) => Some(ident),
                _ => None
            }
        })
        .collect();

    // This will end up looking like [[A], [A, B], [A, B, C], [A, B, C, D], ...]
    let mut vec_collection: Vec<Vec<Ident>> = vec![];
    let mut aggregator = vec![];

    while !idents.is_empty() {
        aggregator.push(idents.remove(0));

        vec_collection.push(aggregator.clone());
    }
    
    fn prepend_to_ident(ident: &Ident, prepend: &str) -> Ident {
        let mut new_ident = ident.to_string();
        new_ident.insert_str(0, prepend);
        Ident::new(&new_ident, Span2::call_site())
    }

    let variants = vec_collection.iter().map(|v| {
        let prepended_idents: Vec<Ident> = v.iter().map(|ident| prepend_to_ident(ident, "T")).collect();
        let prepended_futures: Vec<Ident> = v.iter().map(|ident| Ident::new(&format!("T{}O", ident), Span2::call_site())).collect();
        let top_level_type = prepend_to_ident(v.get(0).unwrap(), "T");
        let top_level_future = Ident::new(&format!("T{}O", v.get(0).unwrap()), Span2::call_site());

        let mut i = 1;

        // Generate the Fn where clauses
        let mut where_clauses = vec![];
        // Generate the async Fn where clauses
        let mut async_where_clauses = vec![];
        // Generate the types for the asynf Fns
        let mut future_clauses = vec![];
        while i < v.len() {
            let function_name = v.get(i - 1).unwrap();
            let current_type = prepend_to_ident(function_name, "T");
            let next_type = prepend_to_ident(v.get(i).unwrap(), "T");
            let future_name = Ident::new(&format!("T{function_name}O"), Span2::call_site());
            let next_future = Ident::new(&format!("T{}O", v.get(i).unwrap()), Span2::call_site());
            where_clauses.push(quote! {
                #function_name: Fn(#current_type, Box<dyn Fn(#next_type) -> #next_type>) -> #current_type
            });
            async_where_clauses.push(quote! {
                #function_name: Fn(#current_type, Box<dyn Fn(#next_type) -> #next_future + Send + Sync + 'static>) -> #future_name + Send + Sync + 'static
            });
            future_clauses.push(quote! {
                #future_name: Future<Output=#current_type> + Send
            });
            i += 1;
        }
        let last_function = v.last().unwrap();
        let last_type = prepend_to_ident(last_function, "T");
        let last_future = Ident::new(&format!("T{last_function}O"), Span2::call_site());
        where_clauses.push(quote! {
            #last_function: Fn(#last_type, Box<dyn Fn(#last_type) -> #last_type>) -> #last_type
        });
        async_where_clauses.push(quote! {
            #last_function: Fn(#last_type, Box<dyn Fn(#last_type) -> #last_type + Send + Sync + 'static>) -> #last_future + Send + Sync + 'static
        });
        future_clauses.push(quote! {
            #last_future: Future<Output=#last_type> + Send
        });

        // Generate the nested functions
        let mut nested = quote! {
            Box::new(|v| v)
        };
        for i in (0..v.len()).rev() {
            let index = Literal::usize_unsuffixed(i);
            nested = quote! {
                Box::new(move |v| (self.#index)(v, #nested))
            };  
        }

        // Put the pieces together
        quote! {
            impl<#(#v: 'static + Send + Copy),*, #(#prepended_idents),*> Into<ChainableFn<#top_level_type, #top_level_type, (#(#prepended_idents),*,)>> for (#(#v),*,)
                where
                    #(#where_clauses),*
            {
                fn into(self) -> ChainableFn<#top_level_type, #top_level_type, (#(#prepended_idents),*,)> {
                    ChainableFn::new(#nested)
                }
            }

            impl<#(#v: 'static + Send + Copy),*, #(#prepended_idents),*, #(#prepended_futures),*> Into<ChainableFn<#top_level_type, #top_level_future, (#(#prepended_idents),*, #(#prepended_futures),*,)>> for (#(#v),*,)
                where
                    #(#async_where_clauses),*,
                    #(#future_clauses),*
            {
                fn into(self) -> ChainableFn<#top_level_type, #top_level_future, (#(#prepended_idents),*, #(#prepended_futures),*,)> {
                    ChainableFn::new(#nested)
                }
            }
        }
    });
    
    let gen = quote! {
        struct ChainableFn<T, TO, InnerType> {
            f: Box<dyn Fn(T) -> TO>,
            inner: std::marker::PhantomData<InnerType>,
        }
        
        impl<T, TO, InnerType> ChainableFn<T, TO, InnerType> {
            fn new(f: Box<dyn Fn(T) -> TO>) -> ChainableFn<T, TO, InnerType> {
                ChainableFn {
                    f,
                    inner: std::marker::PhantomData,
                }
            }
        }

        #(#variants)*
    };

    // use rust_format::{Config, Formatter, PostProcess, RustFmt};
    // let config = Config::new_str()
    // .post_proc(PostProcess::ReplaceMarkersAndDocBlocks);
    // proc_macro::Span::call_site()
    //     .note("Thruster code output")
    //     // .note(gen.to_string())
    //     .note(RustFmt::from_config(config).format_tokens(gen.clone()).unwrap())
    //     .emit();

    gen.into()
}
