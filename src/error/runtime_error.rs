use std::fmt;

use crate::token::Token;

use super::object_error::ObjectError;

#[derive(PartialEq)]
pub(crate) enum RuntimeErrorType {
    ObjectError(ObjectError),
    UndefinedVariable(String),
}

impl RuntimeErrorType {
    fn msg(&self) -> String {
        match self {
            RuntimeErrorType::ObjectError(e) => e.to_string(),
            RuntimeErrorType::UndefinedVariable(name) => format!("Undefined variable `{}`", name),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct RuntimeError {
    line: usize,
    error_type: RuntimeErrorType,
}

impl RuntimeError {
    pub fn undefined_variable(token: &Token) -> Self {
        Self {
            line: token.line(),
            error_type: RuntimeErrorType::UndefinedVariable(token.lexeme().to_string()),
        }
    }
}

impl From<(usize, ObjectError)> for RuntimeError {
    fn from(value: (usize, ObjectError)) -> Self {
        Self {
            line: value.0,
            error_type: RuntimeErrorType::ObjectError(value.1),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[line {}]: RuntimeError: {}",
            self.line,
            self.error_type.msg()
        )
    }
}

impl fmt::Debug for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl std::error::Error for RuntimeError {}
