use std::fmt;

use super::object_error::ObjectError;

#[derive(PartialEq)]
pub(crate) enum RuntimeErrorType {
    ObjectError(ObjectError),
}

impl RuntimeErrorType {
    fn msg(&self) -> String {
        match self {
            RuntimeErrorType::ObjectError(e) => e.to_string(),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct RuntimeError {
    line: usize,
    error_type: RuntimeErrorType,
}

impl From<(usize, ObjectError)> for RuntimeError {
    fn from(value: (usize, ObjectError)) -> Self {
        Self {
            line: value.0,
            error_type: RuntimeErrorType::ObjectError(value.1),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[line {}]: RuntimeError: {}",
            self.line,
            self.error_type.msg()
        )
    }
}

impl fmt::Debug for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl std::error::Error for RuntimeError {}
