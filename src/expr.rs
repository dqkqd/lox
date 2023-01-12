use crate::{
    token::{Number, Token},
    visitor::Visitor,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Object {
    Null,
    Number(Number),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
    Binary(Binary),
    Unary(Unary),
    Literal(Object),
    Grouping(Grouping),
}

impl Expr {
    pub fn walk_epxr<T>(&self, visitor: &mut impl Visitor<T>) -> T {
        visitor.visit_expr(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Binary {
    pub fn new(left: Expr, operator: Token, right: Expr) -> Self {
        Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

impl Unary {
    pub fn new(operator: Token, right: Expr) -> Self {
        Unary {
            operator,
            right: Box::new(right),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Grouping {
    pub expr: Box<Expr>,
}

impl Grouping {
    pub fn new(expr: Expr) -> Self {
        Grouping {
            expr: Box::new(expr),
        }
    }
}
