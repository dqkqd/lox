use std::fmt;

use super::{parse_error::ParseError, runtime_error::RuntimeError, syntax_error::SyntaxError};

#[derive(PartialEq)]
pub(crate) enum LoxErrorType {
    ParseError(ParseError),
    SyntaxError(SyntaxError),
    RuntimeError(RuntimeError),
}

impl LoxErrorType {
    fn msg(&self) -> String {
        match self {
            LoxErrorType::SyntaxError(e) => format!("SyntaxError: {}", e.msg()),
            LoxErrorType::ParseError(e) => format!("ParseError: {}", e.msg()),
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

impl From<SyntaxError> for LoxError {
    fn from(error: SyntaxError) -> Self {
        Self {
            line: error.line(),
            error_type: LoxErrorType::SyntaxError(error),
        }
    }
}

impl From<ParseError> for LoxError {
    fn from(error: ParseError) -> Self {
        Self {
            line: error.line(),
            error_type: LoxErrorType::ParseError(error),
        }
    }
}

impl From<RuntimeError> for LoxError {
    fn from(error: RuntimeError) -> Self {
        Self {
            line: error.line(),
            error_type: LoxErrorType::RuntimeError(error),
        }
    }
}
