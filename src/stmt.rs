use crate::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stmt {
    Expression(Expr),
    Print(Expr),
}

impl Stmt {
    // pub fn walk_epxr<T>(&self, visitor: &mut impl Visitor<T>) -> T {
    //     visitor.visit_expr(self)
    // }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Expression {
    pub expression: Expr,
}

impl Expression {
    pub fn new(expression: Expr) -> Self {
        Expression { expression }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Print {
    pub expression: Expr,
}

impl Print {
    pub fn new(expression: Expr) -> Self {
        Print { expression }
    }
}
