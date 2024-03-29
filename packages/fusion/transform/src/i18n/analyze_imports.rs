use std::{cell::RefCell, rc::Rc};

use swc_core::{
    ecma::{
        ast::*,
        visit::{
            as_folder, noop_visit_mut_type, noop_visit_type, Fold, Visit, VisitMut, VisitWith,
        },
    },
    plugin::errors::HANDLER,
};
use tracing::debug;

use super::State;
use crate::Config;

pub fn i18n_analyze_imports(config: Rc<Config>, state: Rc<RefCell<State>>) -> impl VisitMut + Fold {
    as_folder(AsAnalyzer { config, state })
}

struct AsAnalyzer {
    config: Rc<Config>,
    state: Rc<RefCell<State>>,
}

impl VisitMut for AsAnalyzer {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, p: &mut Module) {
        let mut v: Analyzer<'_> = Analyzer {
            config: &self.config,
            state: &mut self.state.borrow_mut(),
        };

        p.visit_with(&mut v);
    }

    fn visit_mut_script(&mut self, p: &mut Script) {
        let mut v = Analyzer {
            config: &self.config,
            state: &mut self.state.borrow_mut(),
        };

        p.visit_with(&mut v);
    }
}

pub fn find_id_attribute(opening_element: &JSXOpeningElement) -> Option<String> {
    for attr_or_spread in &opening_element.attrs {
        match attr_or_spread {
            JSXAttrOrSpread::JSXAttr(attr) => {
                match &attr.name {
                    JSXAttrName::Ident(ident) => {
                        if ident.sym.as_ref() == "id" {
                            match &attr.value {
                                Some(JSXAttrValue::Lit(lit)) => {
                                    match &lit {
                                        Lit::Str(lit_str) => {
                                            // `LitStr` has a `value` field that is a `JsWord` type
                                            // which also needs to be converted to a string for use
                                            return Some(lit_str.value.as_ref().to_string());
                                        }
                                        _ => HANDLER.with(|handler| {
                                            handler
                                                .struct_span_err(
                                                    attr.span,
                                                    "The translate component id prop must be a a \
                                                     string literal.",
                                                )
                                                .emit();
                                        }),
                                    }
                                }
                                _ => HANDLER.with(|handler| {
                                    handler
                                        .struct_span_err(
                                            attr.span,
                                            "The translate component id prop must be a a string \
                                             literal.",
                                        )
                                        .emit();
                                }),
                            }
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
    None
}

struct Analyzer<'a> {
    config: &'a Config,
    state: &'a mut State,
}

fn get_var_name(var_declarator: &VarDeclarator) -> Option<String> {
    match &var_declarator.name {
        Pat::Ident(binding_ident) => {
            let name = binding_ident.id.sym.as_ref().to_owned();
            Some(name)
        }
        _ => None,
    }
}

impl Visit for Analyzer<'_> {
    noop_visit_type!();

    fn visit_var_declarator(&mut self, var_declarator: &VarDeclarator) {
        var_declarator.visit_children_with(self);
        if let Some(name) = get_var_name(var_declarator) {
            if let Some(init_val) = var_declarator.init.as_ref() {
                match &**init_val {
                    Expr::Call(call_expr) => match &call_expr.callee {
                        Callee::Expr(boxed_expr) => match &**boxed_expr {
                            Expr::Ident(ident) => {
                                if ident.sym.as_ref() == "useTranslations" {
                                    self.state.add_use_translation_alias(name);
                                }
                            }
                            _ => (),
                        },
                        _ => (),
                    },
                    _ => (),
                }
            }
        }
    }

    fn visit_jsx_opening_element(&mut self, opening_element: &JSXOpeningElement) {
        opening_element.visit_children_with(self);
        match &opening_element.name {
            JSXElementName::Ident(ident) => {
                if self
                    .state
                    .get_fusion_plugin_imports()
                    .contains(ident.sym.as_ref())
                {
                    let attr_id_value = find_id_attribute(opening_element);
                    match attr_id_value {
                        Some(id) => {
                            self.state.add_translation_id(id);
                        }
                        None => (),
                    }
                }
            }
            _ => (),
        }
    }

    fn visit_import_decl(&mut self, i: &ImportDecl) {
        let is_i18n = if self.config.top_level_import_paths.is_empty() {
            &*i.src.value == "fusion-plugin-i18n-react"
                || i.src.value.starts_with("fusion-plugin-i18n-react/")
        } else {
            self.config.top_level_import_paths.contains(&i.src.value)
        };

        if is_i18n {
            for s in &i.specifiers {
                match s {
                    ImportSpecifier::Named(s) => {
                        let import_name = s
                            .imported
                            .as_ref()
                            .map(|v| match v {
                                ModuleExportName::Ident(v) => &*v.sym,
                                ModuleExportName::Str(v) => &*v.value,
                            })
                            .unwrap_or(&*s.local.sym);
                        if import_name == "Translate"
                            || import_name == "useTranslations"
                            || import_name == "withTranslations"
                        {
                            self.state
                                .add_fusion_plugin_import(s.local.sym.as_ref().to_owned());
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    fn visit_call_expr(&mut self, call_expr: &CallExpr) {
        call_expr.visit_children_with(self);
        let error_msg = "The withTranslations HoC must be called with an array of string literal \
                         translation keys.";
        match &call_expr.callee {
            Callee::Expr(boxed_expr) => match &**boxed_expr {
                Expr::Ident(ident) => {
                    if self
                        .state
                        .get_fusion_plugin_imports()
                        .contains(ident.sym.as_ref())
                    {
                        debug!("withTranslations call matched");
                        if let Some(first_arg) = call_expr.args.get(0) {
                            match &*first_arg.expr {
                                Expr::Array(array_lit) => {
                                    for elem in &array_lit.elems {
                                        if let Some(ExprOrSpread { expr, .. }) = elem {
                                            if let Expr::Lit(Lit::Str(str_lit)) = &**expr {
                                                self.state.add_translation_id(
                                                    str_lit.value.as_ref().to_owned(),
                                                );
                                            } else {
                                                HANDLER.with(|handler| {
                                                    handler
                                                        .struct_span_err(call_expr.span, error_msg)
                                                        .emit();
                                                });
                                            }
                                        }
                                    }
                                }
                                _ => HANDLER.with(|handler| {
                                    handler.err(error_msg);
                                }),
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
