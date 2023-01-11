use crate::token::{Number, Token};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
    Binary(Binary),
    Unary(Unary),
    Literal(Literal),
    Grouping(Grouping),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Binary {
    left: Box<Expr>,
    operator: Token,
    right: Box<Expr>,
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
    operator: Token,
    right: Box<Expr>,
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
pub(crate) enum Literal {
    Null,
    Number(Number),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Grouping {
    expr: Box<Expr>,
}

impl Grouping {
    pub fn new(expr: Expr) -> Self {
        Grouping {
            expr: Box::new(expr),
        }
    }
}
