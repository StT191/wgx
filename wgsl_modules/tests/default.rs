
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

    tokens_eq!(composed.code.as_ref(), concatenated);
}


#[test]
fn including_from_path() {

    let composed = wgsl_modules::include!("../shaders/shader_all.wgsl");
    let concatenated = include_str!("../shaders/concatenated.wgsl");

    tokens_eq!(composed, concatenated);
}


#[test] #[should_panic]
fn circular_includes() {
    Module::load_from_path("shaders/circular.wgsl").unwrap();
}

#[test] #[should_panic]
fn nonexistent_includes() {
    Module::load_from_path("shaders/nonexistent.wgsl").unwrap();
}

#[test] #[should_panic]
fn invalid_path() {
    Module::load_from_path("").unwrap();
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
        &import normal_2d from "inline::util"
    }).unwrap();

    tokens_eq!(module.code.as_ref().trim(), include_str!("../shaders/util.wgsl"));
}


#[test]
fn inline_registering() {

    register!("inline/util" <= {
        fn normal_2d(v:vec2<f32>) -> vec2<f32> {
            return vec2<f32>(v.y, -v.x);
        }
    });

    let module_src = register!("module/module" <= {
        &import * from "../inline/util"
    });

    tokens_eq!(module_src, include_str!("../shaders/util.wgsl"));
}


#[test]
fn inline_include() {

    let module_src = register!("module" <= {
        &import * from "wgsl_modules/shaders/util.wgsl"
    });

    tokens_eq!(module_src, include_str!("../shaders/util.wgsl"));
}