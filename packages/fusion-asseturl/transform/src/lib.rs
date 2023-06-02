#![deny(unused)]

use std::{cell::RefCell, rc::Rc};

use serde::Deserialize;
use swc_core::{
    common::chain,
    ecma::{
        atoms::JsWord,
        visit::{Fold, VisitMut},
    },
};

pub use crate::{
    utils::{analyze, analyzer, State},
    visitors::asseturl::asseturl,
};

mod utils;
mod visitors;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    #[serde(default = "true_by_default")]
    pub display_name: bool,

    #[serde(default = "true_by_default")]
    pub ssr: bool,

    #[serde(default = "true_by_default")]
    pub file_name: bool,

    #[serde(default = "default_index_file_name")]
    pub meaningless_file_names: Vec<String>,

    #[serde(default)]
    pub namespace: String,

    #[serde(default)]
    pub top_level_import_paths: Vec<JsWord>,

    #[serde(default)]
    pub transpile_template_literals: bool,

    #[serde(default)]
    pub minify: bool,

    #[serde(default)]
    pub pure: bool,

    #[serde(default = "true_by_default")]
    pub css_prop: bool,
}

fn true_by_default() -> bool {
    true
}

fn default_index_file_name() -> Vec<String> {
    vec!["index".to_string()]
}

impl Config {}

pub fn asseturl_macro(config: Config) -> impl Fold + VisitMut {
    let state: Rc<RefCell<State>> = Default::default();
    let config = Rc::new(config);

    chain!(analyzer(config.clone(), state.clone()), asseturl(state))
}
