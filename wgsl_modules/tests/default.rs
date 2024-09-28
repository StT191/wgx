#![feature(assert_matches)]

use std::assert_matches::assert_matches;
use wgsl_modules::{Module, ModuleCache, inline};
use proc_macro2::TokenStream;
use std::str::FromStr;


// helper to test equality of source code
macro_rules! tokens_eq {
    ($left:expr, $right:expr) => {
        assert_eq!(
            format!("{}", TokenStream::from_str($left).unwrap()),
            format!("{}", TokenStream::from_str($right).unwrap())
        );
    }
}


#[test]
fn loading_from_path() {

    let composed = Module::load_from_path("shaders/shader_all.wgsl").unwrap();
    let concatenated = include_str!("../shaders/concatenated.wgsl");

    tokens_eq!(composed.code(), concatenated);
}


#[test]
fn including_from_path() {

    let composed = wgsl_modules::include!("../shaders/shader_all.wgsl");
    let concatenated = include_str!("../shaders/concatenated.wgsl");

    tokens_eq!(composed, concatenated);
}


#[test]
fn circular_includes() {

    let res = Module::load_from_path("../wgsl_modules/shaders/circular.wgsl");

    assert_matches!(res, Err(err) if err.to_string().starts_with("circular dependency"));
}


#[test]
fn nonexistent_includes() {

    let res = Module::load_from_path("shaders/nonexistent.wgsl");

    assert_matches!(res, Err(err) if err.to_string().starts_with("failed loading module from path"));
}


#[test]
fn invalid_path() {

    let res = Module::load_from_path("");

    assert_matches!(res, Err(err) if err.to_string().starts_with("invalid path"));
}


#[test]
fn inline_loading_into_cache() {

    let mut modules = ModuleCache::new();

    modules.load("inline::util", stringify!{
        fn normal_2d(v:vec2f) -> vec2f {
            return vec2f(v.y, -v.x);
        }
    }).unwrap();

    let module = modules.load("inline::module", stringify!{
        &include "inline::util";
    }).unwrap();

    tokens_eq!(module.code(), include_str!("../shaders/util.wgsl"));
}


#[test]
fn inline_registering() {

    inline!("$inline//$util" <= {
        fn normal_2d(v:vec2f) -> vec2f {
            return vec2f(v.y, -v.x);
        }
    });

    let module_src = inline!("$module/$module" <= {
        &include "../$inline/$util";
    });

    tokens_eq!(module_src, include_str!("../shaders/util.wgsl"));
}


#[test]
fn inline_including() {

    let module_src = inline!("$module" <= {
        &include "../shaders/util.wgsl";
    });

    tokens_eq!(module_src, stringify!{
        fn normal_2d(v:vec2f) -> vec2f { return vec2f(v.y, -v.x); }
    });
}


mod inner;

#[test]
fn inline_inner_including() {

    let module_src = inline!("$module" <= {
        &include "./inner/$src";
    });

    tokens_eq!(module_src, stringify!{
        fn normal_2d(v:vec2f) -> vec2f { return vec2f(v.y, -v.x); }
    });
}


#[test]
fn naga_parsing_failing() {

    let res = Module::load("$module", stringify!{
        nonexistent token;
    }).and_then(|module| {
        // parse naga
        module.naga_module(false)
    });

    assert_matches!(res, Err(err) if err.to_string().starts_with("error: expected global item"));
}


#[test]
fn naga_validation_failing() {

    let res = Module::load("$module", include_str!("../shaders/invalid.wgsl")).and_then(|module| {
        // parse and validate naga
        module.naga_module(true)
    });

    assert_matches!(res, Err(err) if err.to_string().starts_with("error: Entry point vs_main at Vertex is invalid"));
}