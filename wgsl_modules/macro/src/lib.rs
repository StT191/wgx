#![feature(proc_macro_span, track_path, local_key_cell_methods)]

use std::cell::RefCell;
use wgsl_modules_loader::{ModuleCache, load_with_cache};

use proc_macro::{TokenStream, Span, tracked_path};
use syn::{parse_macro_input, LitStr};
use proc_macro2::{Literal, TokenTree, TokenStream as TokenStream2};
use quote::quote;


thread_local!(static CACHE: RefCell<ModuleCache> = ModuleCache::new().into());


#[proc_macro]
pub fn include(input: TokenStream) -> TokenStream {

    let mut path = Span::call_site().source_file().path().parent().unwrap().to_path_buf();

    let include_path: LitStr = parse_macro_input!(input);

    path.push(include_path.value());

    CACHE.with_borrow_mut(|cache| {
        match load_with_cache(cache, &path) {
            Ok(module) => {
                // track source code files
                tracked_path::path(module.path.to_str().unwrap());

                for file_path in &module.dependent_files {
                    tracked_path::path(file_path.to_str().unwrap());
                }

                TokenStream2::from(TokenTree::from(Literal::string(&module.code))).into()
            },
            Err(err) => quote!(compile_error!(#err)).into(),
        }
    })
}