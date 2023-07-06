#![deny(unused)]

use std::{cell::RefCell, rc::Rc};

use serde::Deserialize;
use swc_core::{
    common::{chain, FileName},
    ecma::{
        atoms::JsWord,
        visit::{Fold, VisitMut},
    },
};

pub use crate::{
    asseturl_utils::{analyzer as asseturlAnalyzer, State as asseturlState},
    gql_utils::{analyzer as gqlAnalyzer, State as gqlState},
    i18n::{i18n_analyze_imports, i18n_analyze_use_translation, State as i18n_state},
    visitors::{
        asseturl::asseturl, dirname::dirname, gql::gql, i18n::i18n_report_ids, split::split,
    },
};

mod asseturl_utils;
mod gql_utils;
mod i18n;
mod shared;
mod visitors;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    #[serde(default = "true_by_default")]
    pub ssr: bool,

    #[serde(default = "true_by_default")]
    pub transform_asseturl: bool,

    #[serde(default = "true_by_default")]
    pub transform_gql: bool,

    #[serde(default = "true_by_default")]
    pub transform_i18n: bool,

    #[serde(default = "true_by_default")]
    pub transform_split: bool,

    #[serde(default = "true_by_default")]
    pub transform_dirname: bool,

    #[serde(default)]
    pub top_level_import_paths: Vec<JsWord>,
}

fn true_by_default() -> bool {
    true
}

impl Config {}

pub fn i18n_macro(config: Config, file_name: FileName) -> impl Fold + VisitMut {
    let state: Rc<RefCell<i18n_state>> = Default::default();
    let config = Rc::new(config);
    chain!(
        i18n_analyze_imports(config.clone(), state.clone()),
        i18n_analyze_use_translation(state.clone()),
        i18n_report_ids(file_name.clone(), state)
    )
}

pub fn asseturl_macro(config: Config) -> impl Fold + VisitMut {
    let state: Rc<RefCell<asseturlState>> = Default::default();
    let config = Rc::new(config);
    chain!(
        asseturlAnalyzer(config.clone(), state.clone()),
        asseturl(state)
    )
}

pub fn gql_macro(config: Config) -> impl Fold + VisitMut {
    let state: Rc<RefCell<gqlState>> = Default::default();
    let config = Rc::new(config);
    chain!(gqlAnalyzer(config.clone(), state.clone()), gql(state))
}

pub fn dirname_macro(file_name: FileName) -> impl Fold + VisitMut {
    dirname(file_name)
}

pub fn split_macro(config: Config, file_name: FileName) -> impl Fold + VisitMut {
    let config = Rc::new(config);
    split(config.clone(), file_name)
}
