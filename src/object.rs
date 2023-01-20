use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::{
    callable::{Callable, LoxCallable},
    error::object_error::ObjectError,
};

type ObjectOperationResult = Result<Object, ObjectError>;

pub(crate) type Number = f64;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Object {
    Null,
    Number(Number),
    String(String),
    Bool(bool),
    Callable(LoxCallable),
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

    pub fn gt(&self, rhs: &Self) -> ObjectOperationResult {
        let lhs = self.as_number().ok_or_else(ObjectError::comparision)?;
        let rhs = rhs.as_number().ok_or_else(ObjectError::comparision)?;
        Ok(Object::Bool(lhs > rhs))
    }

    pub fn lt(&self, rhs: &Self) -> ObjectOperationResult {
        let lhs = self.as_number().ok_or_else(ObjectError::comparision)?;
        let rhs = rhs.as_number().ok_or_else(ObjectError::comparision)?;
        Ok(Object::Bool(lhs < rhs))
    }

    pub fn ge(&self, rhs: &Self) -> ObjectOperationResult {
        let lhs = self.as_number().ok_or_else(ObjectError::comparision)?;
        let rhs = rhs.as_number().ok_or_else(ObjectError::comparision)?;
        Ok(Object::Bool(lhs >= rhs))
    }

    pub fn le(&self, rhs: &Self) -> ObjectOperationResult {
        let lhs = self.as_number().ok_or_else(ObjectError::comparision)?;
        let rhs = rhs.as_number().ok_or_else(ObjectError::comparision)?;
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
            Object::Callable(fun) => format!("<fn {}>", fun.name()),
        }
    }
}

impl Neg for Object {
    type Output = ObjectOperationResult;
    fn neg(self) -> Self::Output {
        let result = self.as_number().ok_or_else(ObjectError::negative)?;
        Ok(Object::Number(-result))
    }
}

impl Add for Object {
    type Output = ObjectOperationResult;
    fn add(self, rhs: Self) -> Self::Output {
        if self.is_number() && rhs.is_number() {
            Ok(Object::Number(
                self.as_number().unwrap() + rhs.as_number().unwrap(),
            ))
        } else if self.is_string() && rhs.is_string() {
            Ok(Object::String(self.to_string() + &rhs.to_string()))
        } else {
            Err(ObjectError::addition())
        }
    }
}

impl Sub for Object {
    type Output = ObjectOperationResult;
    fn sub(self, rhs: Self) -> Self::Output {
        let lhs = self.as_number().ok_or_else(ObjectError::subtract)?;
        let rhs = rhs.as_number().ok_or_else(ObjectError::subtract)?;
        Ok(Object::Number(lhs - rhs))
    }
}

impl Mul for Object {
    type Output = ObjectOperationResult;
    fn mul(self, rhs: Self) -> Self::Output {
        let lhs = self.as_number().ok_or_else(ObjectError::multiplication)?;
        let rhs = rhs.as_number().ok_or_else(ObjectError::multiplication)?;
        Ok(Object::Number(lhs * rhs))
    }
}

impl Div for Object {
    type Output = ObjectOperationResult;
    fn div(self, rhs: Self) -> Self::Output {
        let lhs = self.as_number().ok_or_else(ObjectError::division)?;
        let rhs = rhs.as_number().ok_or_else(ObjectError::division)?;
        if rhs == 0.0 {
            Err(ObjectError::zero_division())
        } else {
            Ok(Object::Number(lhs / rhs))
        }
    }
}
