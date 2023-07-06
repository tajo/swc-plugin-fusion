#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::path::PathBuf;

use fusion::Config;
use swc_common::{plugin::metadata::TransformPluginMetadataContextKind, FileName};
use swc_core::{
    ecma::{ast::Program, visit::VisitMutWith},
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

#[plugin_transform]
fn fusion_asseturl(mut program: Program, data: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<Config>(
        &data
            .get_transform_plugin_config()
            .expect("failed to get plugin config for swc-plugin-fusion"),
    )
    .expect("invalid config for swc=plugin-fusion");

    let file_name =
        if let Some(filename) = data.get_context(&TransformPluginMetadataContextKind::Filename) {
            FileName::Real(PathBuf::from(filename))
        } else {
            FileName::Anon
        };

    if config.transform_asseturl {
        let mut pass = fusion::asseturl_macro(config.clone());
        program.visit_mut_with(&mut pass);
    }

    if config.transform_gql {
        let mut pass = fusion::gql_macro(config.clone());
        program.visit_mut_with(&mut pass);
    }

    if config.transform_dirname {
        let mut pass = fusion::dirname_macro(file_name.clone());
        program.visit_mut_with(&mut pass);
    }

    if config.transform_split {
        let mut pass = fusion::split_macro(config.clone(), file_name.clone());
        program.visit_mut_with(&mut pass);
    }

    if config.transform_i18n {
        let mut pass = fusion::i18n_macro(config.clone(), file_name.clone());
        program.visit_mut_with(&mut pass);
    }

    program
}
