use std::{
    hash::{Hash, Hasher},
    num::ParseFloatError,
    ops::{Add, Deref, DerefMut, Div, Mul, Neg, Sub},
    str::FromStr,
};

use crate::{callable::LoxCallable, class::LoxInstance, error::object_error::ObjectError};

type ObjectOperationResult = Result<Object, ObjectError>;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct Number(f64);

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state)
    }
}

impl Add for Number {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Number(self.0 + rhs.0)
    }
}

impl Sub for Number {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Number(self.0 - rhs.0)
    }
}

impl Mul for Number {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Number(self.0 * rhs.0)
    }
}

impl Div for Number {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Number(self.0 / rhs.0)
    }
}

impl Neg for Number {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Number(-self.0)
    }
}

impl Deref for Number {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Number {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl FromStr for Number {
    type Err = ParseFloatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<f64>().map(Number)
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) enum Object {
    Null,
    Number(Number),
    String(String),
    Bool(bool),
    Callable(LoxCallable),
    LoxInstance(LoxInstance),
}

#[allow(dead_code)]
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
            Object::Callable(callable) => callable.to_string(),
            Object::LoxInstance(instance) => instance.to_string(),
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
        if rhs == Number(0.0) {
            Err(ObjectError::zero_division())
        } else {
            Ok(Object::Number(lhs / rhs))
        }
    }
}
