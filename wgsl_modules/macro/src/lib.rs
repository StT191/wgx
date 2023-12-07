#![feature(proc_macro_span, track_path)]

use std::cell::RefCell;
use wgsl_modules_loader::ModuleCache;

use proc_macro::{TokenStream, TokenTree, Literal, Span, tracked_path};
use syn::{parse_macro_input, LitStr};
use quote::quote;


thread_local!(static CACHE: RefCell<ModuleCache> = ModuleCache::new().into());


#[proc_macro]
pub fn include(input: TokenStream) -> TokenStream {

    let mut path = Span::call_site().source_file().path().parent().unwrap().to_path_buf();

    let include_path: LitStr = parse_macro_input!(input);

    path.push(include_path.value());

    CACHE.with_borrow_mut(|cache| {
        match cache.load_from_path(&path) {
            Ok(module) => {
                // track source code files
                tracked_path::path(path.to_str().unwrap());

                for file_path in &module.dependencies {
                    // may be put in cache manually
                    if file_path.exists() {
                        tracked_path::path(file_path.to_str().unwrap());
                    }
                }

                TokenTree::from(Literal::string(&module.code)).into()
            },
            Err(err) => quote!(compile_error!(#err)).into(),
        }
    })
}



use quote::quote_spanned;
use syn::token::Le;
use proc_macro::{Delimiter};


// get the next token or return error
macro_rules! next {
    ($span:ident, $input:ident) => {
        if let Some(token) = $input.next() {
            #[allow(unused_assignments)]
            let _ = { $span = token.span() }; // make it a stmt to use the attribute
            token
        }
        else {
            return quote_spanned!{$span.into()=>compile_error!("unexpected end of input")}.into()
        }
    }
}


#[proc_macro]
pub fn register(input: TokenStream) -> TokenStream {

    let mut input = input.into_iter();
    let mut span = Span::call_site();

    // parse path
    let path_token = next!(span, input).into();
    let path = parse_macro_input!(path_token as LitStr).value();

    // parse le
    let le_token = TokenStream::from_iter([next!(span, input), next!(span, input)]);
    parse_macro_input!(le_token as Le);

    // parse source
    let source_token = next!(span, input);

    let source = match &source_token {
        TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
            format!("{}", group.stream())
        },
        _ => {
            let source_token = source_token.into();
            parse_macro_input!(source_token as LitStr).value()
        },
    };

    CACHE.with_borrow_mut(|cache| {
        match cache.load(path, source) {
            Ok(module) => {
                // track possible source code files
                for file_path in &module.dependencies {
                    // may be put in cache manually
                    if file_path.exists() {
                        println!("{:?}", file_path);
                        tracked_path::path(file_path.to_str().unwrap());
                    }
                }

                TokenTree::from(Literal::string(&module.code)).into()
            },
            Err(err) => quote!(compile_error!(#err)).into(),
        }
    })
}