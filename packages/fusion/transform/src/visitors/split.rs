use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use swc_core::{
    common::{errors::HANDLER, FileName, DUMMY_SP},
    ecma::{
        ast::*,
        visit::{as_folder, noop_visit_mut_type, Fold, VisitMut},
    },
};

pub fn split(file_name: FileName) -> impl VisitMut + Fold {
    as_folder(DisplayNameAndId { file_name })
}

#[derive(Debug)]
struct DisplayNameAndId {
    file_name: FileName,
}

impl VisitMut for DisplayNameAndId {
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

                                let encoded_file_name = utf8_percent_encode(
                                    &self.file_name.to_string(),
                                    NON_ALPHANUMERIC,
                                )
                                .to_string();

                                let encoded_import = utf8_percent_encode(
                                    &lit_str.value.clone().to_string(),
                                    NON_ALPHANUMERIC,
                                )
                                .to_string();

                                let module_id = format!(
                                    "{}{}{}{}",
                                    "virtual:fusion-vite-split-loader?importer=",
                                    encoded_file_name,
                                    "&specifier=",
                                    encoded_import
                                );

                                let obj_lit = Expr::Object(ObjectLit {
                                                span: DUMMY_SP,
                                                props: vec![
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
                                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                        key: PropName::Ident(Ident::new("__FUSION_DYNAMIC_IMPORT_METADATA__".into(), DUMMY_SP)),
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
                                                    }))),
                                                ],
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
