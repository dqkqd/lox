use crate::{expr::Expr, token::Token, visitor::Visitor};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Var),
    Block(Block),
}

impl Stmt {
    pub fn walk_stmt<E, S>(&self, visitor: &mut impl Visitor<E, S>) -> S {
        visitor.visit_stmt(self)
    }
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Var {
    pub identifier: Token,
    pub expression: Expr,
}

impl Var {
    pub fn new(identifier: Token, expression: Expr) -> Self {
        Self {
            identifier,
            expression,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Block {
    pub statements: Vec<Stmt>,
}

impl Block {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }
}
