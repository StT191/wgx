#![feature(track_path)]

use std::{cell::RefCell, path::{Path, PathBuf}};
use wgsl_modules_loader::{Module, ModuleCache, naga::valid::{ValidationFlags, Capabilities}};

use proc_macro::{TokenStream, TokenTree, Literal, Span, tracked_path};
use syn::{parse_macro_input, LitStr};
use quote::quote;

use anyhow::{Result as Res};


thread_local!(static CACHE: RefCell<ModuleCache> = ModuleCache::new().into());


// helper
fn handle_result(res: Res<&Module>, path: &Path) -> TokenStream {
    match res.and_then(|module| {
        // validate naga_module
        module.naga_module(Some((ValidationFlags::all(), Capabilities::all())))?;
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
        Err(err) => {
            let err = format!("{err:?}");
            quote!(compile_error!(#err)).into()
        },
    }
}



#[proc_macro]
pub fn include(input: TokenStream) -> TokenStream {

    let dir_path = PathBuf::from(Span::call_site().file()).parent().unwrap().to_owned();
    let path = dir_path.join(parse_macro_input!(input as LitStr).value());

    CACHE.with_borrow_mut(|cache| {
        handle_result(cache.load_from_path(&path), &path)
    })
}



use quote::quote_spanned;
use syn::{token::Le, parse::{self, ParseBuffer, Error}};
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

    let dir_path = PathBuf::from(Span::call_site().file()).parent().unwrap().to_owned();

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
            group.stream().to_string()
        },
        _ => {
            let parse_lit_str = |input: &ParseBuffer<'_>| -> parse::Result<String> {
                if input.peek(LitStr) {
                    let lit_str: LitStr = input.parse()?;
                    if lit_str.suffix() == "" { Ok(lit_str.value()) }
                    else { Err(Error::new(lit_str.span(), "unexpected suffix")) }
                }
                else { Err(input.error("expected block or string literal")) }
            };
            let source_token = source_token.into();
            parse_macro_input!(source_token with parse_lit_str)
        },
    };

    // assert end of input
    if let Some(token) = input.next() {
        return quote_spanned!{token.span().into()=>compile_error!("unexpected token")}.into()
    }

    CACHE.with_borrow_mut(|cache| {
        handle_result(cache.load(&path, source), &path)
    })
}