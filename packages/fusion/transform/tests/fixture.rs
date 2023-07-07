#![deny(unused)]

use std::{fs::read_to_string, path::PathBuf};

use fusion::{asseturl_macro, dirname_macro, gql_macro, i18n_macro, split_macro, Config};
use swc_core::{
    common::{chain, FileName},
    ecma::{
        parser::{EsConfig, Syntax},
        transforms::{testing::test_fixture},
        visit::{Fold, VisitMut},
    },
};

pub trait FoldVisitMut: Fold + VisitMut {}
impl<T: Fold + VisitMut> FoldVisitMut for T {}

fn chain_plugins(plugins: Vec<Box<dyn FoldVisitMut + 'static>>) -> Box<dyn FoldVisitMut + 'static> {
    plugins
        .into_iter()
        .reduce(|a, b| Box::new(chain!(a, b)) as Box<dyn FoldVisitMut>)
        .expect("Expected at least one plugin")
}

#[testing::fixture("../../../fixtures/**/code.js")]
fn fixture(input: PathBuf) {
    let dir = input.parent().unwrap();
    let config = read_to_string(dir.join("config.json")).expect("failed to read config.json");
    println!("---- Config -----\n{}", config);
    let config: Config = serde_json::from_str(&config).unwrap();
    let file_name = FileName::Real(PathBuf::from("/path/to/file.js"));

    test_fixture(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        &|_| {
            let mut plugins: Vec<Box<dyn FoldVisitMut + 'static>> = Vec::new();
            if config.transform_asseturl {
                plugins.push(Box::new(asseturl_macro(config.clone())));
            }
            if config.transform_gql {
                plugins.push(Box::new(gql_macro(config.clone())));
            }
            if config.transform_dirname {
                plugins.push(Box::new(dirname_macro(file_name.clone())));
            }
            if config.transform_split {
                plugins.push(Box::new(split_macro(config.clone(), file_name.clone())));
            }
            if config.transform_i18n {
                plugins.push(Box::new(i18n_macro(config.clone(), file_name.clone())));
            }
            chain_plugins(plugins)
        },
        &input,
        &dir.join("output.js"),
        Default::default(),
    )
}
