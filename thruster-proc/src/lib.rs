// #![feature(proc_macro_diagnostic)]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span as Span2, TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::quote;

#[proc_macro]
pub fn m(items: TokenStream) -> TokenStream {
    let items = proc_macro2::TokenStream::from(items);

    let idents = items
        .into_iter()
        .filter(|v| matches!(v, TokenTree2::Ident(_)))
        .clone();
    let pointers = idents.clone().into_iter().map(|_| {
        quote! {
            MiddlewareFnPointer<_>
        }
    });

    let gen = quote! {
        {
            use thruster::parser::middleware_traits::{MiddlewareFnPointer, MiddlewareTuple, ToTuple};

            let val: (#( #pointers),*,) = (#( #idents ),*,);
            val.to_tuple()
        }
    };

    // proc_macro::Span::call_site()
    //     .note("Thruster code output")
    //     .note(gen.to_string())
    //     .emit();

    gen.into()
}

#[proc_macro]
pub fn async_middleware(items: TokenStream) -> TokenStream {
    let items = proc_macro2::TokenStream::from(items);

    let mut item_iter = items.into_iter();

    item_iter.next();
    item_iter.next();

    let items = match item_iter.next() {
        Some(TokenTree2::Group(g)) => g.stream(),
        _ => panic!("Second item should be a group."),
    };

    let idents = items
        .into_iter()
        .filter(|v| matches!(v, TokenTree2::Ident(_)))
        .clone();
    let pointers = idents.clone().into_iter().map(|_| {
        quote! {
            MiddlewareFnPointer<_>
        }
    });

    let gen = quote! {
        {
            use thruster::parser::middleware_traits::{MiddlewareFnPointer, MiddlewareTuple, ToTuple};

            let val: (#( #pointers),*,) = (#( #idents ),*,);
            val.to_tuple()
        }
    };

    // proc_macro::Span::call_site()
    //     .note("Thruster code output")
    //     .note(gen.to_string())
    //     .emit();

    gen.into()
}

#[proc_macro_attribute]
pub fn middleware_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let syn::Item::Fn(mut function_item) = syn::parse(item.clone()).unwrap() {
        let name = function_item.ident.clone();
        let new_name = Ident::new(&format!("__async_{}", name), Span2::call_site());
        function_item.ident = new_name.clone();

        let visibility = function_item.vis.clone();
        let arguments = function_item.decl.inputs.clone();
        let generics = function_item.decl.generics.clone();

        let context_type = match &arguments[0] {
            syn::FnArg::Captured(cap) => &cap.ty,
            _ => panic!("Expected the first argument to be a context type"),
        };
        let new_return_type = Ident::new(
            &format!("__MiddlewareReturnValue_{}", name),
            Span2::call_site(),
        );
        let crate_path = match attr.to_string().as_str() {
            "_internal" => quote! {
                crate::core::{ MiddlewareReturnValue as #new_return_type }
            },
            _ => quote! {
                thruster::{ MiddlewareReturnValue as #new_return_type }
            },
        };

        let gen = quote! {
            #function_item

            use #crate_path;
            #visibility fn #name#generics(ctx: #context_type, next: MiddlewareNext<#context_type>) -> #new_return_type<#context_type> {
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

#[proc_macro]
pub fn generate_tuples(items: TokenStream) -> TokenStream {
    let items = proc_macro2::TokenStream::from(items);

    let mut idents: Vec<Ident> = items
        .into_iter()
        .filter(|v| matches!(v, TokenTree2::Ident(_)))
        .map(|v| {
            if let TokenTree2::Ident(i) = v {
                i
            } else {
                panic!("Should never get here.")
            }
        })
        .collect();
    let ident_count = idents.len();

    let mut vec_collection: Vec<Vec<Ident>> = vec![];
    let mut aggregator = vec![];

    while !idents.is_empty() {
        aggregator.push(idents.remove(0));

        vec_collection.push(aggregator.clone());
    }

    let mut enum_variants = vec![];
    let mut to_tuple_variants = vec![];
    let mut from_tuple_variants = vec![];
    for i in 0..vec_collection.len() {
        let idents = vec_collection.get(i).unwrap();
        let last_a = idents.last().unwrap();
        let last_b = idents.last().unwrap();
        let last_d = idents.last().unwrap();

        let values_a: Vec<TokenStream2> = idents
            .iter()
            .map(|_v| {
                quote! {
                    M<T>
                }
            })
            .collect();
        let values_b: Vec<Ident> = idents
            .iter()
            .map(|v| Ident::new(&format!("{}", v).to_lowercase(), Span2::call_site()))
            .collect();
        let values_c = values_b.clone();
        let values_e = values_a.clone();
        let values_f = values_b.clone();
        let values_g = values_b.clone();

        enum_variants.push(quote! {
            #last_a(#(#values_a),*)
        });

        to_tuple_variants.push(quote! {
            MiddlewareTuple::#last_b(#(#values_b),*) => (#(#values_c),*,)
        });

        from_tuple_variants.push(quote! {
            #[allow(unused_parens)]
            impl<T: 'static + Sync + Send> ToTuple<T> for (#(#values_e),*,) {
                fn to_tuple(self) -> MiddlewareTuple<T> {
                    #[allow(non_snake_case)]
                    let (#(#values_f),*,) = self;

                    MiddlewareTuple::#last_d(#(#values_g),*)
                }
            }
        });
    }

    const VALUES: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut combine_outter = vec![];
    for i in 0..vec_collection.len() {
        let idents = vec_collection.get(i).unwrap();
        let last = idents.last().unwrap();

        let values_a: Vec<Ident> = idents
            .iter()
            .map(|v| Ident::new(&format!("{}", v).to_lowercase(), Span2::call_site()))
            .collect();

        let mut inner_values = vec![];
        for j in 0..vec_collection.len() {
            let inner_idents = vec_collection.get(j).unwrap();

            let outter = Ident::new(&format!("{}", last).to_lowercase(), Span2::call_site());
            let last = inner_idents.last().unwrap();

            let values: Vec<Ident> = inner_idents
                .iter()
                .map(|v| {
                    Ident::new(
                        &format!("{}_{}", outter, v).to_lowercase(),
                        Span2::call_site(),
                    )
                })
                .collect();

            let count_usize = i + j + 2;
            let count = proc_macro2::Literal::usize_suffixed(count_usize);
            if count_usize <= ident_count {
                let output_variant = Ident::new(
                    &format!("{}", VALUES.chars().nth(count_usize - 1).unwrap()),
                    Span2::call_site(),
                );
                let values_c = values_a.clone();
                let values_d = values.clone();
                let values_e = values.clone();

                inner_values.push(quote! {
                    MiddlewareTuple::#last(#(#values_d),*) => MiddlewareTuple::#output_variant(#(#values_c),*, #(#values_e),*)
                });
            } else {
                inner_values.push(quote! {
                    MiddlewareTuple::#last(#(#values),*) => panic!("Can't handle {}-tuple", #count)
                });
            }
        }

        combine_outter.push(quote! {
            MiddlewareTuple::#last(#(#values_a),*) => {
                match other {
                    #(#inner_values),*
                }
            }
        });
    }

    let gen = quote! {
        #[derive(Clone, Debug)]
        pub enum MiddlewareTuple<T> {
            #(
                #enum_variants
            ),*
        }

        pub trait ToTuple<T> {
            fn to_tuple(self) -> MiddlewareTuple<T>;
        }

        impl<T: Send + Sync> MiddlewareTuple<T> {
            pub fn combine(self, other: MiddlewareTuple<T>) -> MiddlewareTuple<T> {
                match self {
                    #(
                        #combine_outter
                    ),*
                }
            }
        }

        impl<T: 'static + Send + Sync> IntoMiddleware<T, M<T>> for MiddlewareTuple<T> {
            fn middleware(self) -> Box<dyn Fn(T) -> Pin<Box<dyn Future<Output = Result<T, ThrusterError<T>>> + Send + Sync>> + Send + Sync> {
                match self {
                    #(
                        #to_tuple_variants.middleware()
                    ),*
                }
            }
        }


        #(#from_tuple_variants)*
    };

    // proc_macro::Span::call_site()
    //     .note("Thruster code output")
    //     .note(gen.to_string())
    //     .emit();

    gen.into()
}
