use std::fmt;

#[derive(PartialEq)]
pub(crate) enum LoxErrorType {
    // scanner error
    UnterminatedString,

    // parser error
    ExpectedExpression,
    UnexpectedCharacter(char),
    UnexpectedToken(String),
    ParserExpectToken(String, String),
}

impl LoxErrorType {
    fn msg(&self) -> String {
        match self {
            LoxErrorType::UnterminatedString => "Unterminated string".to_string(),
            LoxErrorType::UnexpectedCharacter(c) => {
                format!("Unexpected character `{}`", c)
            }
            LoxErrorType::ParserExpectToken(found, expected) => {
                format!("Expected `{}`. Found `{}`.", expected, found)
            }
            LoxErrorType::UnexpectedToken(found) => {
                format!("Unexpected token `{}`.", found)
            }
            LoxErrorType::ExpectedExpression => "Expected expression".to_string(),
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
