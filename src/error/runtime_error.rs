use std::fmt;

use crate::{object::Object, token::Token};

use super::object_error::ObjectError;

#[derive(PartialEq)]
pub(crate) enum RuntimeErrorType {
    ObjectError(ObjectError),
    UndefinedVariable(String),
    WriteError(String),
    NumberArgumentsMismatch(usize, usize),
    ReturnValue(Object), // this is not error
}

impl RuntimeErrorType {
    fn msg(&self) -> String {
        match self {
            RuntimeErrorType::ObjectError(e) => e.to_string(),
            RuntimeErrorType::UndefinedVariable(name) => format!("Undefined variable `{}`", name),
            RuntimeErrorType::WriteError(err) => err.to_string(),
            RuntimeErrorType::NumberArgumentsMismatch(paramc, argc) => {
                format!("Expected {} arguments. Found {} arguments", paramc, argc)
            }
            RuntimeErrorType::ReturnValue(_) => unreachable!("this should not be called as error"),
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

    pub fn number_arguments_mismatch(line: usize, params_count: usize, args_count: usize) -> Self {
        Self {
            line,
            error_type: RuntimeErrorType::NumberArgumentsMismatch(params_count, args_count),
        }
    }

    pub fn return_value(line: usize, value: Object) -> Self {
        Self {
            line,
            error_type: RuntimeErrorType::ReturnValue(value),
        }
    }

    pub fn get_value_from_return(self) -> Object {
        match self.error_type {
            RuntimeErrorType::ReturnValue(object) => object,
            _ => Object::Null,
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

impl From<std::io::Error> for RuntimeError {
    fn from(value: std::io::Error) -> Self {
        Self {
            line: 0,
            error_type: RuntimeErrorType::WriteError(value.to_string()),
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
