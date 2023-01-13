use std::{
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
};

type ObjectOperationResult = Result<Object, ObjectError>;

pub(crate) type Number = f64;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Object {
    Null,
    Number(Number),
    String(String),
    Bool(bool),
}

impl Object {
    pub fn as_null(&self) -> Option<Object> {
        match self {
            Object::Null => Some(Object::Null),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    pub fn as_number(&self) -> Option<Number> {
        match self {
            Object::Number(number) => Some(*number),
            _ => None,
        }
    }

    pub fn is_number(&self) -> bool {
        self.as_number().is_some()
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Object::String(string) => Some(string.clone()),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        self.as_string().is_some()
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Object::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Null => false,
            Object::Bool(b) => *b,
            _ => true,
        }
    }

    pub fn ge(&self, rhs: &Self) -> ObjectOperationResult {
        let lhs = self
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::ComparisionError))?;
        let rhs = rhs
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::ComparisionError))?;
        Ok(Object::Bool(lhs > rhs))
    }

    pub fn le(&self, rhs: &Self) -> ObjectOperationResult {
        let lhs = self
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::ComparisionError))?;
        let rhs = rhs
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::ComparisionError))?;
        Ok(Object::Bool(lhs < rhs))
    }

    pub fn gt(&self, rhs: &Self) -> ObjectOperationResult {
        let lhs = self
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::ComparisionError))?;
        let rhs = rhs
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::ComparisionError))?;
        Ok(Object::Bool(lhs >= rhs))
    }

    pub fn lt(&self, rhs: &Self) -> ObjectOperationResult {
        let lhs = self
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::ComparisionError))?;
        let rhs = rhs
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::ComparisionError))?;
        Ok(Object::Bool(lhs <= rhs))
    }

    pub fn ne(&self, rhs: &Self) -> ObjectOperationResult {
        Ok(Object::Bool(self != rhs))
    }

    pub fn eq(&self, rhs: &Self) -> ObjectOperationResult {
        Ok(Object::Bool(self == rhs))
    }
}

impl ToString for Object {
    fn to_string(&self) -> String {
        match self {
            Object::Null => "".to_string(),
            Object::Number(number) => number.clone().to_string(),
            Object::String(string) => string.clone(),
            Object::Bool(b) => b.to_string(),
        }
    }
}

impl Neg for Object {
    type Output = ObjectOperationResult;
    fn neg(self) -> Self::Output {
        let result = self
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::NegativeError))?;
        Ok(Object::Number(result))
    }
}

impl Add for Object {
    type Output = ObjectOperationResult;
    fn add(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Ok(Object::Number(
                self.as_number().unwrap() + rhs.as_number().unwrap(),
            ))
        } else {
            Ok(Object::String(self.to_string() + &rhs.to_string()))
        }
    }
}

impl Sub for Object {
    type Output = ObjectOperationResult;
    fn sub(self, rhs: Self) -> Self::Output {
        let lhs = self
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::SubtractError))?;
        let rhs = rhs
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::SubtractError))?;
        Ok(Object::Number(lhs - rhs))
    }
}

impl Mul for Object {
    type Output = ObjectOperationResult;
    fn mul(self, rhs: Self) -> Self::Output {
        let lhs = self
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::MultipleError))?;
        let rhs = rhs
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::MultipleError))?;
        Ok(Object::Number(lhs * rhs))
    }
}

impl Div for Object {
    type Output = ObjectOperationResult;
    fn div(self, rhs: Self) -> Self::Output {
        let lhs = self
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::DivisionError))?;
        let rhs = rhs
            .as_number()
            .ok_or_else(|| ObjectError::new(ObjectErrorType::DivisionError))?;
        Ok(Object::Number(lhs / rhs))
    }
}

#[derive(PartialEq)]
pub(crate) enum ObjectErrorType {
    // simple error for all case
    Error(&'static str),

    ComparisionError,
    NegativeError,
    SubtractError,
    MultipleError,
    DivisionError,
}

impl ObjectErrorType {
    fn msg(&self) -> String {
        match self {
            ObjectErrorType::Error(s) => s.to_string(),
            ObjectErrorType::ComparisionError => {
                "Could not compare non-number together".to_string()
            }
            ObjectErrorType::NegativeError => "Could not negative non-number".to_string(),
            ObjectErrorType::SubtractError => "Could not subtract non-number".to_string(),
            ObjectErrorType::MultipleError => "Could not multiply non-number".to_string(),
            ObjectErrorType::DivisionError => "Could not divide non-number".to_string(),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct ObjectError {
    error_type: ObjectErrorType,
}

impl ObjectError {
    pub fn new(error_type: ObjectErrorType) -> Self {
        Self { error_type }
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
