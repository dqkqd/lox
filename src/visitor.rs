use crate::{expr::Expr, stmt::Stmt};

pub(crate) trait Visitor<E, S> {
    fn visit_expr(&mut self, e: &Expr) -> E;
    fn visit_stmt(&mut self, s: &Stmt) -> S;
}
