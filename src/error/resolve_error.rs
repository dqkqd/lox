use std::fmt;

use crate::{source::CharPos, stmt::Return, token::Token};

use super::reporter::impl_error_pos;

#[derive(PartialEq)]
pub(crate) enum ResolveErrorType {
    ReadDuringInitializer(String),
    VarAlreadyExistInScope(String),
    ReturnFromTopLevel,
    ReturnInsideInit,
    CallThisOutsideClass,
}

impl ResolveErrorType {
    fn msg(&self) -> String {
        match self {
            ResolveErrorType::ReadDuringInitializer(name) => {
                format!("Couldn't read `{name}` in its own initializer")
            }
            ResolveErrorType::VarAlreadyExistInScope(name) => {
                format!("Already a variable `{name}` in this scope.")
            }
            ResolveErrorType::ReturnFromTopLevel => {
                "Could not return from top level code".to_string()
            }
            ResolveErrorType::CallThisOutsideClass => {
                "Could not use `this` outside of a class".to_string()
            }
            ResolveErrorType::ReturnInsideInit => "Could not return inside constructor".to_string(),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct ResolveError {
    start_pos: CharPos,
    end_pos: CharPos,
    error_type: ResolveErrorType,
}

impl_error_pos!(ResolveError);

impl ResolveError {
    pub fn read_during_initializer(token: &Token) -> Self {
        Self {
            start_pos: token.start_pos(),
            end_pos: token.end_pos(),
            error_type: ResolveErrorType::ReadDuringInitializer(token.lexeme().to_string()),
        }
    }

    pub fn already_declared(token: &Token) -> Self {
        Self {
            start_pos: token.start_pos(),
            end_pos: token.end_pos(),
            error_type: ResolveErrorType::VarAlreadyExistInScope(token.lexeme().to_string()),
        }
    }

    pub fn return_from_top_level(return_expr: &Return) -> Self {
        Self {
            start_pos: return_expr.keyword.start_pos(),
            end_pos: return_expr.semicolon.end_pos(),
            error_type: ResolveErrorType::ReturnFromTopLevel,
        }
    }

    pub fn return_inside_init(return_expr: &Return) -> Self {
        Self {
            start_pos: return_expr.keyword.start_pos(),
            end_pos: return_expr.semicolon.end_pos(),
            error_type: ResolveErrorType::ReturnInsideInit,
        }
    }

    pub fn call_this_outside_class(token: &Token) -> Self {
        Self {
            start_pos: token.start_pos(),
            end_pos: token.end_pos(),
            error_type: ResolveErrorType::CallThisOutsideClass,
        }
    }
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[line {}]: ResolveError: {}",
            self.start_pos.line + 1,
            self.error_type.msg()
        )
    }
}

impl fmt::Debug for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl std::error::Error for ResolveError {}
