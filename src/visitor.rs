use crate::expr::Expr;

pub(crate) trait Visitor<T> {
    fn visit_expr(&mut self, e: &Expr) -> T;
}
