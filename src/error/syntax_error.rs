use std::fmt;

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
    pub fn line(&self) -> usize {
        self.line
    }

    pub fn msg(&self) -> String {
        self.error_type.msg()
    }

    pub fn unterminated_string(line: usize) -> Self {
        Self {
            line,
            error_type: SyntaxErrorType::UnterminatedString,
        }
    }

    pub fn unexpected_character(line: usize, ch: char) -> Self {
        Self {
            line,
            error_type: SyntaxErrorType::UnexpectedCharacter(ch),
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}]: {}", self.line, self.error_type.msg())
    }
}

impl fmt::Debug for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl std::error::Error for SyntaxError {}
