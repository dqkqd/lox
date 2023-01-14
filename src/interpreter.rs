use crate::{
    error::{lox_error::LoxError, runtime_error::RuntimeError},
    expr::Expr,
    object::Object,
    token::TokenType,
    visitor::Visitor,
};

#[derive(Default)]
pub(crate) struct Interpreter;

type InterpreterResult = Result<Object, LoxError>;

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
                let line = binary.operator.line();
                match binary.operator.token_type() {
                    TokenType::Minus => {
                        Ok((lhs - rhs).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::Star => {
                        Ok((lhs * rhs).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::Slash => {
                        Ok((lhs / rhs).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::Plus => {
                        Ok((lhs + rhs).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::Greater => {
                        Ok((lhs.ge(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::GreaterEqual => {
                        Ok((lhs.gt(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::Less => {
                        Ok((lhs.le(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::LessEqual => {
                        Ok((lhs.lt(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::BangEqual => {
                        Ok((lhs.ne(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::EqualEqual => {
                        Ok((lhs.eq(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    _ => unimplemented!(),
                }
            }
            Expr::Unary(unary) => {
                let rhs = self.visit_expr(&unary.right)?;
                let line = unary.operator.line();
                match unary.operator.token_type() {
                    TokenType::Minus => Ok((-rhs).map_err(|err| RuntimeError::from((line, err)))?),
                    TokenType::Bang => Ok(Object::Bool(!rhs.is_truthy())),
                    _ => unimplemented!(),
                }
            }
            Expr::Literal(object) => Ok(object.clone()),
            Expr::Grouping(group) => Ok(self.visit_expr(&group.expr)?),
        }
    }
}
