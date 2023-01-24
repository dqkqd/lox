use std::fmt;

use crate::{object::Object, source::CharPos, token::Token};

use super::{object_error::ObjectError, reporter::impl_error_pos};

#[derive(PartialEq)]
pub(crate) enum RuntimeErrorType {
    ObjectError(ObjectError),
    UndefinedVariable(String),
    WriteError(String),
    NumberArgumentsMismatch(usize, usize),
    ObjectNotCallable(String),
    ReturnValue(Object), // this is not error
    OnlyClassInstanceHasField(String, String),
    UndefinedProperty(String),
}

impl RuntimeErrorType {
    fn msg(&self) -> String {
        match self {
            RuntimeErrorType::ObjectError(e) => e.to_string(),
            RuntimeErrorType::UndefinedVariable(name) => format!("Undefined variable `{name}`"),
            RuntimeErrorType::WriteError(err) => err.to_string(),
            RuntimeErrorType::NumberArgumentsMismatch(paramc, argc) => {
                format!("Expected {paramc} arguments. Found {argc} arguments")
            }
            RuntimeErrorType::ReturnValue(_) => unreachable!("this should not be called as error"),
            RuntimeErrorType::ObjectNotCallable(name) => format!("`{name}` is not a function"),
            RuntimeErrorType::OnlyClassInstanceHasField(object, field) => {
                format!("`{object}` is not class instance. It cannot have field `{field}`")
            }
            RuntimeErrorType::UndefinedProperty(property) => {
                format!("Undefined property `{property}`")
            }
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct RuntimeError {
    start_pos: CharPos,
    end_pos: CharPos,
    error_type: RuntimeErrorType,
}

impl_error_pos!(RuntimeError);

impl RuntimeError {
    pub fn undefined_variable(token: &Token) -> Self {
        Self {
            start_pos: token.start_pos(),
            end_pos: token.end_pos(),
            error_type: RuntimeErrorType::UndefinedVariable(token.lexeme().to_string()),
        }
    }

    pub fn number_arguments_mismatch(
        token: &Token,
        params_count: usize,
        args_count: usize,
    ) -> Self {
        Self {
            start_pos: token.start_pos(),
            end_pos: token.end_pos(),
            error_type: RuntimeErrorType::NumberArgumentsMismatch(params_count, args_count),
        }
    }

    pub fn object_not_callable(token: &Token, object: &Object) -> Self {
        Self {
            start_pos: token.start_pos(),
            end_pos: token.end_pos(),
            error_type: RuntimeErrorType::ObjectNotCallable(object.to_string()),
        }
    }

    pub fn only_class_instance_has_field(object: &Object, field: &Token) -> Self {
        Self {
            start_pos: field.start_pos(),
            end_pos: field.end_pos(),
            error_type: RuntimeErrorType::OnlyClassInstanceHasField(
                object.to_string(),
                field.lexeme().to_string(),
            ),
        }
    }

    pub fn undefined_property(property: &Token) -> Self {
        Self {
            start_pos: property.start_pos(),
            end_pos: property.end_pos(),
            error_type: RuntimeErrorType::UndefinedProperty(property.lexeme().to_string()),
        }
    }

    pub fn return_value(token: &Token, value: Object) -> Self {
        Self {
            start_pos: token.start_pos(),
            end_pos: token.end_pos(),
            error_type: RuntimeErrorType::ReturnValue(value),
        }
    }

    pub fn get_value_from_return(self) -> Object {
        match self.error_type {
            RuntimeErrorType::ReturnValue(object) => object,
            _ => Object::Null,
        }
    }
}

impl From<(&Token, ObjectError)> for RuntimeError {
    fn from(value: (&Token, ObjectError)) -> Self {
        Self {
            start_pos: value.0.start_pos(),
            end_pos: value.0.end_pos(),
            error_type: RuntimeErrorType::ObjectError(value.1),
        }
    }
}

impl From<std::io::Error> for RuntimeError {
    fn from(value: std::io::Error) -> Self {
        Self {
            start_pos: CharPos::default(),
            end_pos: CharPos::default(),
            error_type: RuntimeErrorType::WriteError(value.to_string()),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[line {}]: RuntimeError: {}",
            self.start_pos.line + 1,
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
