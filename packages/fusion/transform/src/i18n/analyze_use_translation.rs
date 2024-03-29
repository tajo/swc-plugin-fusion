use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use swc_core::{
    ecma::{
        ast::*,
        visit::{
            as_folder, noop_visit_mut_type, noop_visit_type, Fold, Visit, VisitMut, VisitWith,
        },
    },
    plugin::errors::HANDLER,
};

use super::State;

pub fn i18n_analyze_use_translation(state: Rc<RefCell<State>>) -> impl VisitMut + Fold {
    as_folder(AsAnalyzer { state })
}

struct AsAnalyzer {
    state: Rc<RefCell<State>>,
}

impl VisitMut for AsAnalyzer {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, p: &mut Module) {
        let mut v: Analyzer<'_> = Analyzer {
            state: &mut self.state.borrow_mut(),
        };

        p.visit_with(&mut v);
    }

    fn visit_mut_script(&mut self, p: &mut Script) {
        let mut v = Analyzer {
            state: &mut self.state.borrow_mut(),
        };

        p.visit_with(&mut v);
    }
}

struct Analyzer<'a> {
    state: &'a mut State,
}

impl Visit for Analyzer<'_> {
    noop_visit_type!();

    fn visit_call_expr(&mut self, call_expr: &CallExpr) {
        call_expr.visit_children_with(self);
        match &call_expr.callee {
            Callee::Expr(boxed_expr) => match &**boxed_expr {
                Expr::Ident(ident) => {
                    if self
                        .state
                        .get_use_translation_alias()
                        .contains(ident.sym.as_ref())
                    {
                        match call_expr.args.first() {
                            Some(arg) => match arg {
                                ExprOrSpread { expr, .. } => match &**expr {
                                    &Expr::Lit(ref lit) => match lit {
                                        Lit::Str(lit_str) => {
                                            self.state.add_translation_id(
                                                lit_str.value.clone().to_string(),
                                            );
                                        }
                                        _ => {}
                                    },
                                    Expr::Tpl(tpl) => {
                                        if tpl.quasis.first().unwrap().raw.clone().to_string()
                                            == ""
                                        {
                                            HANDLER.with(|handler| {
                                                handler
                                                    .struct_span_err(
                                                        call_expr.span,
                                                        "useTranslations template literal must be \
                                                         hinted, e.g. \
                                                         useTranslations(`hint.${{foo}}`) vs \
                                                         useTranslations(`${{foo}}`)",
                                                    )
                                                    .emit();
                                            });
                                        }
                                        let mut tpl_parts: BTreeSet<String> = BTreeSet::new();
                                        for tpl_element in &tpl.quasis {
                                            let lit_str = &tpl_element.raw;
                                            tpl_parts.insert(lit_str.clone().to_string());
                                        }
                                        self.state.add_translation_id_tpl(tpl_parts);
                                    }
                                    _ => HANDLER.with(|handler| {
                                        handler
                                            .struct_span_err(
                                                call_expr.span,
                                                "useTranslations result function must be passed a \
                                                 string literal or hinted template literal",
                                            )
                                            .emit();
                                    }),
                                },
                            },
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
