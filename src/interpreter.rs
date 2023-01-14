use crate::{
    error::object_error::ObjectError, expr::Expr, object::Object, token::TokenType,
    visitor::Visitor,
};

use std::fmt;

#[derive(Default)]
pub(crate) struct Interpreter;

type InterpreterResult = Result<Object, InterpreterError>;

impl Interpreter {
    pub fn expr(&mut self, e: &Expr) -> InterpreterResult {
        e.walk_epxr(self)
    }
}

impl Visitor<InterpreterResult> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> InterpreterResult {
        match e {
            Expr::Binary(binary) => {
                let lhs = self.visit_expr(&binary.left)?;
                let rhs = self.visit_expr(&binary.right)?;
                match binary.operator.token_type() {
                    TokenType::Minus => Ok((lhs - rhs).map_err(InterpreterError::from)?),
                    TokenType::Star => Ok((lhs * rhs).map_err(InterpreterError::from)?),
                    TokenType::Slash => Ok((lhs / rhs).map_err(InterpreterError::from)?),
                    TokenType::Plus => Ok((lhs + rhs).map_err(InterpreterError::from)?),
                    TokenType::Greater => Ok((lhs.ge(&rhs)).map_err(InterpreterError::from)?),
                    TokenType::GreaterEqual => Ok((lhs.gt(&rhs)).map_err(InterpreterError::from)?),
                    TokenType::Less => Ok((lhs.le(&rhs)).map_err(InterpreterError::from)?),
                    TokenType::LessEqual => Ok((lhs.lt(&rhs)).map_err(InterpreterError::from)?),
                    TokenType::BangEqual => Ok((lhs.ne(&rhs)).map_err(InterpreterError::from)?),
                    TokenType::EqualEqual => Ok((lhs.eq(&rhs)).map_err(InterpreterError::from)?),
                    _ => unimplemented!(),
                }
            }
            Expr::Unary(unary) => {
                let rhs = self.visit_expr(&unary.right)?;
                match unary.operator.token_type() {
                    TokenType::Minus => Ok((-rhs).map_err(InterpreterError::from)?),
                    TokenType::Bang => Ok(Object::Bool(!rhs.is_truthy())),
                    _ => unimplemented!(),
                }
            }
            Expr::Literal(object) => Ok(object.clone()),
            Expr::Grouping(group) => Ok(self.visit_expr(&group.expr)?),
        }
    }
}

#[derive(PartialEq)]
pub(crate) enum InterpreterErrorType {
    // simple error for all case
    Error(String),
}

impl InterpreterErrorType {
    fn msg(&self) -> String {
        match self {
            InterpreterErrorType::Error(s) => s.to_string(),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct InterpreterError {
    error_type: InterpreterErrorType,
}

impl InterpreterError {
    pub fn new(error_type: InterpreterErrorType) -> Self {
        Self { error_type }
    }
}

impl From<ObjectError> for InterpreterError {
    fn from(value: ObjectError) -> Self {
        Self {
            error_type: InterpreterErrorType::Error(value.to_string()),
        }
    }
}
impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_type.msg())
    }
}

impl fmt::Debug for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl std::error::Error for InterpreterError {}
