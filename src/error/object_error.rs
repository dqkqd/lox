use std::fmt;

#[derive(PartialEq)]
pub(crate) enum ObjectErrorType {
    ComparisionError,
    NegativeError,
    SubtractError,
    MultiplicationError,
    DivisionError,
}

impl ObjectErrorType {
    fn msg(&self) -> String {
        match self {
            ObjectErrorType::ComparisionError => {
                "Could not compare non-number together".to_string()
            }
            ObjectErrorType::NegativeError => "Could not negative non-number".to_string(),
            ObjectErrorType::SubtractError => "Could not subtract non-number".to_string(),
            ObjectErrorType::MultiplicationError => "Could not multiply non-number".to_string(),
            ObjectErrorType::DivisionError => "Could not divide non-number".to_string(),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct ObjectError {
    error_type: ObjectErrorType,
}

impl ObjectError {
    pub fn comparision_error() -> Self {
        Self {
            error_type: ObjectErrorType::ComparisionError,
        }
    }

    pub fn negative_error() -> Self {
        Self {
            error_type: ObjectErrorType::NegativeError,
        }
    }

    pub fn subtract_error() -> Self {
        Self {
            error_type: ObjectErrorType::SubtractError,
        }
    }

    pub fn multiplication_error() -> Self {
        Self {
            error_type: ObjectErrorType::MultiplicationError,
        }
    }

    pub fn division_error() -> Self {
        Self {
            error_type: ObjectErrorType::DivisionError,
        }
    }
}
impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_type.msg())
    }
}

impl fmt::Debug for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self)
    }
}

impl std::error::Error for ObjectError {}
