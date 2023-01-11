use crate::{
    expr::{Expr, Literal},
    visitor::Visitor,
};

#[derive(Default)]
pub(crate) struct AstPrinter;

impl AstPrinter {
    pub fn print_expr(&mut self, e: &Expr) {
        println!("{}", e.walk_epxr(self))
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Binary(binary) => {
                let left = self.visit_expr(&binary.left);
                let right = self.visit_expr(&binary.right);
                let operator = binary.operator.lexeme();
                format!("({operator} {left} {right})")
            }
            Expr::Unary(unary) => {
                let operator = unary.operator.lexeme();
                let right = self.visit_expr(&unary.right);
                format!("({operator} {right})")
            }
            Expr::Literal(literal) => match literal {
                Literal::Null => "nil".to_string(),
                Literal::Number(n) => (*n).to_string(),
                Literal::String(s) => s.clone(),
                Literal::Bool(b) => b.to_string(),
            },
            Expr::Grouping(group) => {
                let expr = self.visit_expr(&group.expr);
                format!("( {expr} )")
            }
        }
    }
}
