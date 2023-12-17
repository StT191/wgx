#![feature(assert_matches)]

use std::assert_matches::assert_matches;
use wgsl_modules::{Module, ModuleCache, register};
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

    assert_matches!(res, Err(err) if err.starts_with("circular dependency"));
}


#[test]
fn nonexistent_includes() {

    let res = Module::load_from_path("shaders/nonexistent.wgsl");

    assert_matches!(res, Err(err) if err.starts_with("No such file or directory"));
}

#[test]
fn invalid_path() {

    let res = Module::load_from_path("");

    assert_matches!(res, Err(err) if err.starts_with("invalid path"));
}


#[test]
fn inline_loading_into_cache() {

    let mut modules = ModuleCache::new();

    modules.load("inline::util", stringify!{
        fn normal_2d(v:vec2<f32>) -> vec2<f32> {
            return vec2<f32>(v.y, -v.x);
        }
    }).unwrap();

    let module = modules.load("inline::module", stringify!{
        &include "inline::util";
    }).unwrap();

    tokens_eq!(module.code(), include_str!("../shaders/util.wgsl"));
}


#[test]
fn inline_registering() {

    register!("$inline//$util" <= {
        fn normal_2d(v:vec2<f32>) -> vec2<f32> {
            return vec2<f32>(v.y, -v.x);
        }
    });

    let module_src = register!("$module/$module" <= {
        &include "../$inline/$util";
    });

    tokens_eq!(module_src, include_str!("../shaders/util.wgsl"));
}


#[test]
fn inline_including() {

    let module_src = register!("$module" <= {
        &include "../shaders/util.wgsl";
    });

    tokens_eq!(module_src, stringify!{
        fn normal_2d(v:vec2<f32>) -> vec2<f32> { return vec2<f32>(v.y, -v.x); }
    });
}


mod inner;

#[test]
fn inline_inner_including() {

    let module_src = register!("$module" <= {
        &include "./inner/$src";
    });

    tokens_eq!(module_src, stringify!{
        fn normal_2d(v:vec2<f32>) -> vec2<f32> { return vec2<f32>(v.y, -v.x); }
    });
}



#[test]
fn validation_failing() {

    let res = Module::load("$module", stringify!{
        nonexistent token;
    });

    assert_matches!(res, Err(err) if err.starts_with("error: expected global item"));
}