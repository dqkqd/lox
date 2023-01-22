use std::fmt;

use crate::token::Token;

#[derive(PartialEq)]
pub(crate) enum ResolveErrorType {
    ReadDuringInitializer(String),
    VarAlreadyExistInScope(String),
}

impl ResolveErrorType {
    fn msg(&self) -> String {
        match self {
            ResolveErrorType::ReadDuringInitializer(name) => {
                format!("Couldn't read `{}` in its own initializer", name)
            }
            ResolveErrorType::VarAlreadyExistInScope(name) => {
                format!("Already a variable `{}` in this scope.", name)
            }
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct ResolveError {
    line: usize,
    error_type: ResolveErrorType,
}

impl ResolveError {
    pub fn read_during_initializer(token: &Token) -> Self {
        Self {
            line: token.line(),
            error_type: ResolveErrorType::ReadDuringInitializer(token.lexeme().to_string()),
        }
    }

    pub fn already_declared(token: &Token) -> Self {
        Self {
            line: token.line(),
            error_type: ResolveErrorType::VarAlreadyExistInScope(token.lexeme().to_string()),
        }
    }
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[line {}]: ResolveError: {}",
            self.line,
            self.error_type.msg()
        )
    }
}

impl fmt::Debug for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl std::error::Error for ResolveError {}
