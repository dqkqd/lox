use std::fmt;

use crate::{source::CharPos, token::Token};

use super::reporter::impl_error_pos;

#[derive(PartialEq)]
pub(crate) enum ResolveErrorType {
    ReadDuringInitializer(String),
    VarAlreadyExistInScope(String),
    ReturnFromTopLevel,
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
            ResolveErrorType::ReturnFromTopLevel => format!("Could not return from top level code"),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct ResolveError {
    start_pos: CharPos,
    end_pos: CharPos,
    error_type: ResolveErrorType,
}

impl_error_pos!(ResolveError);

impl ResolveError {
    pub fn read_during_initializer(token: &Token) -> Self {
        Self {
            start_pos: token.start_pos(),
            end_pos: token.end_pos(),
            error_type: ResolveErrorType::ReadDuringInitializer(token.lexeme().to_string()),
        }
    }

    pub fn already_declared(token: &Token) -> Self {
        Self {
            start_pos: token.start_pos(),
            end_pos: token.end_pos(),
            error_type: ResolveErrorType::VarAlreadyExistInScope(token.lexeme().to_string()),
        }
    }

    pub fn return_from_top_level(token: &Token) -> Self {
        Self {
            start_pos: token.start_pos(),
            end_pos: token.end_pos(),
            error_type: ResolveErrorType::ReturnFromTopLevel,
        }
    }
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[line {}]: ResolveError: {}",
            self.start_pos.line + 1,
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
