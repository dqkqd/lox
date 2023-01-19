use crate::{expr::Expr, token::Token, visitor::Visitor};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stmt {
    Expression(Expr),
    Print(Expr),
    Function(Function),
    Var(Var),
    Block(Block),
    If(If),
    While(While),
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct If {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

impl If {
    pub fn new(condition: Expr, then_branch: Stmt, else_branch: Option<Stmt>) -> Self {
        Self {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct While {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

impl While {
    pub fn new(condition: Expr, body: Stmt) -> Self {
        Self {
            condition,
            body: Box::new(body),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Function {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Box<Stmt>,
}

impl Function {
    pub fn new(name: Token, params: Vec<Token>, body: Stmt) -> Self {
        Self {
            name,
            params,
            body: Box::new(body),
        }
    }
}
