use std::fmt;

use crate::token::TokenType;

#[derive(PartialEq)]
pub(crate) enum ParseErrorType {
    ExpectedExpression,
    UnexpectedToken(String, String),
    InvalidAssignment,
    MaximumArguments(usize),
}

impl ParseErrorType {
    fn msg(&self) -> String {
        match self {
            ParseErrorType::UnexpectedToken(found, expected) => {
                format!("Expected `{}`. Found `{}`", expected, found)
            }
            ParseErrorType::ExpectedExpression => "Expected expression".to_string(),
            ParseErrorType::InvalidAssignment => "Inavalid assignment target.".to_string(),
            ParseErrorType::MaximumArguments(argc) => {
                format!("Could not have more than {} arguments", argc)
            }
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct ParseError {
    line: usize,
    error_type: ParseErrorType,
    panic_mode: bool,
}

impl ParseError {
    pub fn expected_expression(line: usize) -> Self {
        Self {
            line,
            error_type: ParseErrorType::ExpectedExpression,
            panic_mode: true,
        }
    }

    pub fn unexpected_token(line: usize, found: &TokenType, expected: &TokenType) -> Self {
        Self {
            line,
            error_type: ParseErrorType::UnexpectedToken(found.to_string(), expected.to_string()),
            panic_mode: true,
        }
    }

    pub fn invalid_assignment(line: usize) -> Self {
        Self {
            line,
            error_type: ParseErrorType::InvalidAssignment,
            panic_mode: true,
        }
    }

    pub fn maximum_arguments(line: usize, size: usize) -> Self {
        Self {
            line,
            error_type: ParseErrorType::MaximumArguments(size),
            panic_mode: true,
        }
    }

    pub fn panic(&self) -> bool {
        self.panic_mode
    }

    pub fn without_panic(self) -> Self {
        Self {
            panic_mode: false,
            ..self
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[line {}]: ParseError: {}",
            self.line,
            self.error_type.msg()
        )
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl std::error::Error for ParseError {}
