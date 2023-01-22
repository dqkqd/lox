use crate::{object::Object, token::Token, visitor::Visitor};

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) enum Expr {
    Binary(Binary),
    Unary(Unary),
    Literal(Object),
    Grouping(Grouping),
    Variable(Variable),
    Assign(Assign),
    Logical(Binary),
    Call(Call),
}

impl Eq for Expr {}

impl Expr {
    pub fn walk_epxr<E, S>(&self, visitor: &mut impl Visitor<E, S>) -> E {
        visitor.visit_expr(self)
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) struct Variable {
    pub name: Token,
}

impl Variable {
    pub fn new(name: Token) -> Self {
        Variable { name }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
}

impl Assign {
    pub fn new(name: Token, value: Expr) -> Self {
        Assign {
            name,
            value: Box::new(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) struct Call {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

impl Call {
    pub fn new(callee: Expr, paren: Token, arguments: Vec<Expr>) -> Self {
        Self {
            callee: Box::new(callee),
            paren,
            arguments,
        }
    }
}
