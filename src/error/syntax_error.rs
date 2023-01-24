use std::fmt;

use crate::source::CharPos;

use super::reporter::impl_error_pos;

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
    start_pos: CharPos,
    end_pos: CharPos,
    error_type: SyntaxErrorType,
}

impl_error_pos!(SyntaxError);

impl SyntaxError {
    pub fn unterminated_string(pos: CharPos) -> Self {
        Self {
            start_pos: pos,
            end_pos: pos,
            error_type: SyntaxErrorType::UnterminatedString,
        }
    }

    pub fn unexpected_character(pos: CharPos) -> Self {
        Self {
            start_pos: pos,
            end_pos: pos,
            error_type: SyntaxErrorType::UnexpectedCharacter(pos.ch),
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[line {}]: SyntaxError: {}",
            self.start_pos.line + 1,
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
