use std::fmt;

#[derive(PartialEq)]
pub(crate) enum ParseErrorType {
    ExpectedExpression,
    UnexpectedToken(String),
}

impl ParseErrorType {
    fn msg(&self) -> String {
        match self {
            ParseErrorType::UnexpectedToken(found) => {
                format!("Unexpected token `{}`.", found)
            }
            ParseErrorType::ExpectedExpression => "Expected expression".to_string(),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct ParseError {
    line: usize,
    error_type: ParseErrorType,
}

impl ParseError {
    pub fn line(&self) -> usize {
        self.line
    }

    pub fn msg(&self) -> String {
        self.error_type.msg()
    }

    pub fn expected_expression(line: usize) -> Self {
        Self {
            line,
            error_type: ParseErrorType::ExpectedExpression,
        }
    }

    pub fn unexpected_token(line: usize, found: &str) -> Self {
        Self {
            line,
            error_type: ParseErrorType::UnexpectedToken(found.to_string()),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}]: {}", self.line, self.error_type.msg())
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl std::error::Error for ParseError {}
