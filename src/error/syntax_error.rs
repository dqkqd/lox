use std::fmt;

use crate::source::CharPos;

#[derive(PartialEq)]
enum SyntaxErrorType {
    UnterminatedString,
    UnexpectedCharacter(char),
}

impl SyntaxErrorType {
    fn msg(&self) -> String {
        match self {
            SyntaxErrorType::UnterminatedString => "Unterminated string".to_string(),
            SyntaxErrorType::UnexpectedCharacter(c) => {
                format!("Unexpected character `{}`", c)
            }
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct SyntaxError {
    line: usize,
    error_type: SyntaxErrorType,
}

impl SyntaxError {
    pub fn unterminated_string(pos: CharPos) -> Self {
        Self {
            line: pos.line,
            error_type: SyntaxErrorType::UnterminatedString,
        }
    }

    pub fn unexpected_character(pos: CharPos) -> Self {
        Self {
            line: pos.line,
            error_type: SyntaxErrorType::UnexpectedCharacter(pos.ch),
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[line {}]: SyntaxError: {}",
            self.line + 1,
            self.error_type.msg()
        )
    }
}

impl fmt::Debug for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl std::error::Error for SyntaxError {}
