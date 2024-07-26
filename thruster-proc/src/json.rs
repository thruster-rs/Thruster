use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{FnArg, ItemFn};

#[derive(Debug, FromMeta)]
struct MacroArgs {
    // #[darling(default)]
    // timeout_ms: Option<u16>,
    // path: String,
}

pub fn json_request(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let input = syn::parse_macro_input!(item as ItemFn);

    let _args = match MacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let block = input.block;
    let mut fn_args = input.sig.inputs.clone();

    let _next = fn_args
        .pop()
        .expect("At least context and next arguments are required");
    let context = fn_args
        .pop()
        .expect("At least context and next arguments are required");
    let context_ident = if let FnArg::Typed(arg) = context.value() {
        // Making an assumption here that the last in the pat is the ident.
        arg.pat
            .to_token_stream()
            .into_iter()
            .last()
            .expect("Could not find appropriate identifier when parsing context pattern")
    } else {
        panic!("Received a self arg in place of a context");
    };

    let json = fn_args.into_iter().filter_map(|p| {
        if let FnArg::Typed(arg) = p {
            Some(quote! {
                let #arg = match context.get_json().await {
                    Ok(val) => val,
                    Err(e) => {
                        use thruster::errors::ErrorSet;
                        return Err(thruster::errors::ThrusterError::parsing_error(#context_ident, &format!("{e:#?}")));
                    }
                };
            })
        } else {
            None
        }
    });

    let mut fn_sig = input.sig;

    let mut context_and_next = Punctuated::new();
    // Add the next arg
    context_and_next.insert(0, fn_sig.inputs.pop().unwrap().into_value());
    // Add the context arg
    context_and_next.insert(0, fn_sig.inputs.pop().unwrap().into_value());

    fn_sig.inputs = context_and_next;

    let gen = quote! {
        #[thruster::middleware_fn]
        #fn_sig {
            use thruster::context::context_ext::ContextExt;

            #(#json)*

            #block
        }
    };

    // use rust_format::{Config, Formatter, PostProcess, RustFmt};
    // let config = Config::new_str().post_proc(PostProcess::ReplaceMarkersAndDocBlocks);
    // proc_macro::Span::call_site()
    //     .note("Thruster code output")
    //     .note(
    //         RustFmt::from_config(config)
    //             .format_tokens(gen.clone())
    //             .unwrap_or_else(|_| gen.to_string()),
    //     )
    //     .emit();

    gen.into()
}
