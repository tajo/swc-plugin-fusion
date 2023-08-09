use std::rc::Rc;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use swc_core::{
    common::{errors::HANDLER, FileName, DUMMY_SP},
    ecma::{
        ast::*,
        visit::{as_folder, noop_visit_mut_type, Fold, VisitMut},
    },
};
use tracing::debug;

use crate::Config;

pub fn split(config: Rc<Config>, file_name: FileName) -> impl VisitMut + Fold {
    as_folder(SplitVisitor { config, file_name })
}

// https://url.spec.whatwg.org/#query-percent-encode-set
const QUERY: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'#').add(b'<').add(b'>');
// https://url.spec.whatwg.org/#path-percent-encode-set
const PATH: &AsciiSet = &QUERY.add(b'?').add(b'`').add(b'{').add(b'}');
// https://url.spec.whatwg.org/#userinfo-percent-encode-set
const USERINFO: &AsciiSet = &PATH
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(0x005c)
    .add(0x005d)
    .add(b'^')
    .add(b'|');
// https://url.spec.whatwg.org/#component-percent-encode-set
const COMPONENT: &AsciiSet = &USERINFO.add(b'$').add(0x0025).add(b'&').add(b'+').add(b',');

fn get_member_props(module_id: String, ssr: bool) -> Vec<PropOrSpread> {
    let mut member_props = vec![
        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Ident(Ident::new("__CHUNK_IDS".into(), DUMMY_SP)),
            value: Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(Ident::new("value".into(), DUMMY_SP)),
                    value: Box::new(Expr::Array(ArrayLit {
                        span: DUMMY_SP,
                        elems: vec![],
                    })),
                })))],
            })),
        }))),
        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Ident(Ident::new("__MODULE_ID".into(), DUMMY_SP)),
            value: Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(Ident::new("value".into(), DUMMY_SP)),
                    value: Box::new(Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: module_id.clone().into(),
                        raw: Default::default(),
                    }))),
                })))],
            })),
        }))),
    ];
    if ssr {
        member_props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Ident(Ident::new(
                "__FUSION_DYNAMIC_IMPORT_METADATA__".into(),
                DUMMY_SP,
            )),
            value: Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(Ident::new("value".into(), DUMMY_SP)),
                    value: Box::new(Expr::Object(ObjectLit {
                        span: DUMMY_SP,
                        props: vec![
                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                key: PropName::Ident(Ident::new("version".into(), DUMMY_SP)),
                                value: Box::new(Expr::Lit(Lit::Num(Number {
                                    span: DUMMY_SP,
                                    value: 0.0,
                                    raw: Default::default(),
                                }))),
                            }))),
                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                key: PropName::Ident(Ident::new("moduleId".into(), DUMMY_SP)),
                                value: Box::new(Expr::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: module_id.clone().into(),
                                    raw: Default::default(),
                                }))),
                            }))),
                        ],
                    })),
                })))],
            })),
        }))));
    }
    return member_props;
}

#[derive(Debug)]
struct SplitVisitor {
    file_name: FileName,
    config: Rc<Config>,
}

impl VisitMut for SplitVisitor {
    noop_visit_mut_type!();

    fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
        match &call_expr.callee {
            Callee::Import(_) => match call_expr.args.first() {
                Some(arg) => match arg {
                    ExprOrSpread { expr, .. } => match &**expr {
                        &Expr::Lit(ref lit) => match lit {
                            Lit::Str(lit_str) => {
                                let obj_ident = Expr::Ident(Ident::new("Object".into(), DUMMY_SP));
                                let define_props_ident = MemberProp::Ident(Ident::new(
                                    "defineProperties".into(),
                                    DUMMY_SP,
                                ));
                                let new_callee = Expr::Member(MemberExpr {
                                    span: DUMMY_SP,
                                    obj: Box::new(obj_ident),
                                    prop: define_props_ident,
                                });
                                let import_arg = Expr::Lit(Lit::Str(Str {
                                    value: lit_str.value.clone(),
                                    span: DUMMY_SP,
                                    raw: Default::default(),
                                }));

                                let import_callee =
                                    Expr::Ident(Ident::new("import".into(), DUMMY_SP));
                                let import_call = Expr::Call(CallExpr {
                                    span: DUMMY_SP,
                                    callee: Callee::Expr(Box::new(import_callee)),
                                    args: vec![ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(import_arg.clone()),
                                    }],
                                    type_args: Default::default(),
                                });

                                let encoded_file_name =
                                    utf8_percent_encode(&self.file_name.to_string(), COMPONENT)
                                        .to_string();

                                let encoded_import = utf8_percent_encode(
                                    &lit_str.value.clone().to_string(),
                                    COMPONENT,
                                )
                                .to_string();

                                let module_id = format!(
                                    "{}{}{}{}",
                                    "virtual:fusion-vite-split-loader?importer=",
                                    encoded_file_name,
                                    "&specifier=",
                                    encoded_import
                                );

                                debug!("config: {:?}", self.config);

                                let obj_lit = Expr::Object(ObjectLit {
                                    span: DUMMY_SP,
                                    props: get_member_props(module_id, self.config.ssr.into()),
                                });

                                let new_args = vec![
                                    ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(import_call),
                                    },
                                    ExprOrSpread {
                                        spread: None,
                                        expr: Box::new(obj_lit),
                                    },
                                ];

                                *call_expr = CallExpr {
                                    span: call_expr.span,
                                    callee: Callee::Expr(Box::new(new_callee)),
                                    args: new_args,
                                    type_args: Default::default(),
                                };
                            }
                            _ => HANDLER.with(|handler| {
                                handler.err(&format!(
                                    "Only string literal is supported in dynamic import"
                                ));
                            }),
                        },
                        _ => HANDLER.with(|handler| {
                            handler.err(&format!(
                                "Only string literal is supported in dynamic import"
                            ));
                        }),
                    },
                },
                _ => {}
            },
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_encode_uri_component() {
        assert_eq!(
            utf8_percent_encode("!@#$%^&*()-_+={}[]\\|'\";:+ ,.<>?/~`", COMPONENT,).to_string(),
            "!%40%23%24%25%5E%26*()-_%2B%3D%7B%7D%5B%5D%5C%7C'%22%3B%3A%2B%20%2C.%3C%3E%3F%2F~%60"
        );
    }
}
