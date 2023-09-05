use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::prepend_stmt,
        visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
    },
    plugin::errors::HANDLER,
};
use tracing::{debug, span, Level};

use crate::{gql_utils::State, shared::converters::JsVarConverter};

pub fn gql(state: Rc<RefCell<State>>) -> impl Fold + VisitMut {
    as_folder(GqlVisitor {
        state,
        to_prepend: BTreeSet::new(),
        converter: JsVarConverter::new("gql"),
    })
}

#[derive(Debug, PartialEq, Hash, Eq, Ord, PartialOrd)]
struct Thing {
    file_path: String,
}

#[derive(Debug)]
struct GqlVisitor {
    state: Rc<RefCell<State>>,
    to_prepend: BTreeSet<Thing>,
    converter: JsVarConverter,
}

impl GqlVisitor {
    fn replace_gql_call(&mut self, expr: &mut Expr) {
        if let Expr::Call(call_expr) = expr {
            if let Callee::Expr(callee) = &call_expr.callee {
                if let Expr::Ident(_ident) = &**callee {
                    if let Some(ExprOrSpread {
                        expr: expr_other,
                        spread: None,
                    }) = call_expr.args.get(0)
                    {
                        match &**expr_other {
                            Expr::Lit(Lit::Str(lit_str)) => {
                                let src_str = lit_str.value.to_string();
                                let new_ident = self.converter.ident_from_path(&src_str);
                                *expr = new_ident.into();
                                self.to_prepend.insert(Thing { file_path: src_str });
                            }
                            _ => HANDLER.with(|handler| {
                                handler
                                    .struct_span_err(
                                        call_expr.span,
                                        "gql() argument must be a string literal",
                                    )
                                    .emit();
                            }),
                        }
                    }
                }
            }
        }
    }
}

impl VisitMut for GqlVisitor {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, n: &mut Module) {
        n.visit_mut_children_with(self);

        for i in self.to_prepend.iter() {
            let new_ident = self.converter.ident_from_path(&i.file_path);

            let specifier = ImportSpecifier::Default(ImportDefaultSpecifier {
                span: new_ident.span,
                local: new_ident,
            });

            prepend_stmt(
                &mut n.body,
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    span: DUMMY_SP,
                    specifiers: vec![specifier],
                    src: Box::new(Str {
                        span: DUMMY_SP,
                        value: i.file_path.clone().into(),
                        raw: None,
                    }),
                    type_only: Default::default(),
                    with: Default::default(),
                })),
            );
        }
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        expr.visit_mut_children_with(self);

        let is_gql = match expr {
            Expr::Call(CallExpr {
                callee: Callee::Expr(callee),
                // args,
                ..
            }) => self.state.borrow().is_gql(&*callee),

            _ => false,
        };

        if !is_gql {
            return;
        }
        debug!("Found gql usage");

        let _tracing = if cfg!(debug_assertions) {
            Some(span!(Level::ERROR, "display_name_and_id").entered())
        } else {
            None
        };

        self.replace_gql_call(expr)
    }
}
