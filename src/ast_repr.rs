use crate::{expr::Expr, object::Object, visitor::Visitor};

#[derive(Default)]
pub(crate) struct AstRepr;

impl AstRepr {
    pub fn expr(&mut self, e: &Expr) -> String {
        e.walk_epxr(self)
    }
}

impl Visitor<String> for AstRepr {
    fn visit_expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Binary(binary) => {
                let left = self.visit_expr(&binary.left);
                let right = self.visit_expr(&binary.right);
                let operator = binary.operator.lexeme();
                format!("(binary {operator} {left} {right})")
            }
            Expr::Unary(unary) => {
                let operator = unary.operator.lexeme();
                let right = self.visit_expr(&unary.right);
                format!("(unary {operator} {right})")
            }
            Expr::Literal(object) => match object {
                Object::Null => "nil".to_string(),
                Object::Number(n) => (*n).to_string(),
                Object::String(s) => format!("\"{}\"", s),
                Object::Bool(b) => b.to_string(),
            },
            Expr::Grouping(group) => {
                let expr = self.visit_expr(&group.expr);
                format!("(group {expr})")
            }
        }
    }
}
