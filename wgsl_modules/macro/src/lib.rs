#![feature(proc_macro_span, track_path)]

use std::{cell::RefCell, path::Path};
use wgsl_modules_loader::{Module, ModuleCache, Res};

use proc_macro::{TokenStream, TokenTree, Literal, Span, tracked_path};
use syn::{parse_macro_input, LitStr};
use quote::quote;


thread_local!(static CACHE: RefCell<ModuleCache> = ModuleCache::new().into());


// helper
fn handle_result(res: Res<&Module>, path: &Path) -> TokenStream {
    match res.and_then(|module| {
        // validate naga_module
        module.naga_module(true)?;
        Ok(module)
    }) {
        Ok(module) => {
            // track source code files
            if path.exists() {
                tracked_path::path(path.to_str().unwrap());
            }

            for file_path in module.dependencies() {
                // may be put in cache manually
                if file_path.exists() {
                    tracked_path::path(file_path.to_str().unwrap());
                }
            }

            TokenTree::from(Literal::string(module.code())).into()
        },
        Err(err) => quote!(compile_error!(#err)).into(),
    }
}



#[proc_macro]
pub fn include(input: TokenStream) -> TokenStream {

    let dir_path = Span::call_site().source_file().path().parent().unwrap().to_owned();
    let path = dir_path.join(parse_macro_input!(input as LitStr).value());

    CACHE.with_borrow_mut(|cache| {
        handle_result(cache.load_from_path(&path), &path)
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
pub fn inline(input: TokenStream) -> TokenStream {

    let dir_path = Span::call_site().source_file().path().parent().unwrap().to_owned();

    let mut input = input.into_iter();
    let mut span = Span::call_site();

    // parse path
    let path_token = next!(span, input).into();
    let path = dir_path.join(parse_macro_input!(path_token as LitStr).value());

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
        handle_result(cache.load(&path, source), &path)
    })
}