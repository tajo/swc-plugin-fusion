use std::path::PathBuf;

use swc_core::{
    common::{FileName, DUMMY_SP},
    ecma::{
        ast::*,
        visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
    },
};
use tracing::debug;

pub fn dirname(file_name: FileName) -> impl VisitMut + Fold {
    as_folder(DirnameVisitor { file_name })
}

#[derive(Debug)]
struct DirnameVisitor {
    file_name: FileName,
}

impl VisitMut for DirnameVisitor {
    noop_visit_mut_type!();

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        expr.visit_mut_children_with(self);

        match expr {
            Expr::Ident(i) => match i.sym.as_ref() {
                "__dirname" => {
                    debug!("replacing __dirname in {:?}", self.file_name);
                    let mut buf = PathBuf::from(self.file_name.to_string());
                    buf.pop();
                    *expr = Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: FileName::Real(buf).to_string().into(),
                        raw: None,
                    }))
                }
                "__filename" => {
                    debug!("replacing __filename in {:?}", self.file_name);
                    debug!(target: "dirname", "visit_mut_ident: {:?}", i);
                    *expr = Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: self.file_name.to_string().into(),
                        raw: None,
                    }))
                }
                _ => {}
            },
            _ => {}
        }
    }
}
