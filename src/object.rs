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
}
