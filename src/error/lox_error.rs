use std::fmt;

use super::runtime_error::RuntimeError;

#[derive(PartialEq)]
pub(crate) enum LoxErrorType {
    RuntimeError(RuntimeError),
}

impl LoxErrorType {
    fn msg(&self) -> String {
        match self {
            LoxErrorType::RuntimeError(e) => format!("RuntimeError: {}", e.msg()),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct LoxError {
    line: usize,
    error_type: LoxErrorType,
}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}]: {}", self.line, self.error_type.msg())
    }
}

impl fmt::Debug for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl std::error::Error for LoxError {}

impl From<RuntimeError> for LoxError {
    fn from(error: RuntimeError) -> Self {
        Self {
            line: error.line(),
            error_type: LoxErrorType::RuntimeError(error),
        }
    }
}
