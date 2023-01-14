use std::fmt;

use super::syntax_error::SyntaxError;

#[derive(PartialEq)]
pub(crate) enum LoxErrorType {
    ExpectedExpression,
    UnexpectedToken(String),
    ParserExpectToken(String, String),

    SyntaxError(SyntaxError),
}

impl LoxErrorType {
    fn msg(&self) -> String {
        match self {
            LoxErrorType::ParserExpectToken(found, expected) => {
                format!("Expected `{}`. Found `{}`.", expected, found)
            }
            LoxErrorType::UnexpectedToken(found) => {
                format!("Unexpected token `{}`.", found)
            }
            LoxErrorType::ExpectedExpression => "Expected expression".to_string(),
            LoxErrorType::SyntaxError(s) => format!("SyntaxError: {}", s.msg()),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct LoxError {
    line: usize,
    error_type: LoxErrorType,
}

impl LoxError {
    pub fn new(line: usize, error_type: LoxErrorType) -> Self {
        Self { line, error_type }
    }
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
