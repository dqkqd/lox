use std::fmt;

#[derive(PartialEq)]
pub(crate) enum ObjectErrorType {
    Comparision,
    Negative,
    Subtract,
    Multiplication,
    Division,
}

impl ObjectErrorType {
    fn msg(&self) -> String {
        match self {
            ObjectErrorType::Comparision => "Could not compare non-number together".to_string(),
            ObjectErrorType::Negative => "Could not negative non-number".to_string(),
            ObjectErrorType::Subtract => "Could not subtract non-number".to_string(),
            ObjectErrorType::Multiplication => "Could not multiply non-number".to_string(),
            ObjectErrorType::Division => "Could not divide non-number".to_string(),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct ObjectError {
    error_type: ObjectErrorType,
}

impl ObjectError {
    pub fn comparision() -> Self {
        Self {
            error_type: ObjectErrorType::Comparision,
        }
    }

    pub fn negative() -> Self {
        Self {
            error_type: ObjectErrorType::Negative,
        }
    }

    pub fn subtract() -> Self {
        Self {
            error_type: ObjectErrorType::Subtract,
        }
    }

    pub fn multiplication() -> Self {
        Self {
            error_type: ObjectErrorType::Multiplication,
        }
    }

    pub fn division() -> Self {
        Self {
            error_type: ObjectErrorType::Division,
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
