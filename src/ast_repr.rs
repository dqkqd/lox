use crate::{expr::Expr, object::Object, stmt::Stmt, visitor::Visitor};

#[derive(Default)]
pub(crate) struct AstRepr;

impl AstRepr {
    pub fn expr(&mut self, e: &Expr) -> String {
        e.walk_epxr(self)
    }

    pub fn stmt(&mut self, s: &Stmt) -> String {
        s.walk_stmt(self)
    }
}

impl Visitor<String, String> for AstRepr {
    fn visit_expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Binary(binary) => {
                let left = self.visit_expr(&binary.left);
                let right = self.visit_expr(&binary.right);
                let operator = binary.operator.lexeme();
                format!("Expr::Binary({left} {operator} {right})")
            }
            Expr::Unary(unary) => {
                let operator = unary.operator.lexeme();
                let right = self.visit_expr(&unary.right);
                format!("Expr::Unary({operator} {right})")
            }
            Expr::Literal(object) => match object {
                Object::Null => "nil".to_string(),
                Object::Number(n) => (*n).to_string(),
                Object::String(s) => format!("\"{}\"", s),
                Object::Bool(b) => b.to_string(),
            },
            Expr::Grouping(group) => {
                let expr = self.visit_expr(&group.expr);
                format!("Expr::Group({expr})")
            }
            Expr::Variable(var) => {
                format!("Expr::Variable({})", var.name.lexeme())
            }
            Expr::Assign(assign) => {
                let value = self.visit_expr(&assign.value);
                format!("Expr::Assign({} = {})", assign.name.lexeme(), value)
            }
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) -> String {
        match s {
            Stmt::Expression(e) => {
                let expr = self.visit_expr(e);
                format!("Stmt::Expr({})", expr)
            }
            Stmt::Print(e) => {
                let value = self.visit_expr(e);
                format!("Stmt::Print({})", value)
            }
            Stmt::Var(var) => {
                let name = var.identifier.lexeme();
                let value = self.visit_expr(&var.expression);
                format!("Stmt::Var({} = {})", name, value)
            }
            Stmt::Block(block) => {
                let mut result = String::new();
                result.push_str("Stmt::Block(");
                result.push_str(
                    &block
                        .statements
                        .iter()
                        .map(|s| self.visit_stmt(s))
                        .collect::<Vec<_>>()
                        .join(" "),
                );
                result.push(')');
                result
            }
        }
    }
}
