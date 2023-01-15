use std::collections::HashMap;

use crate::{object::Object, token::Token};

#[derive(Debug, Default)]
pub(crate) struct Environment {
    values: HashMap<String, Object>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, token: &Token) -> Option<&Object> {
        let value = self.values.get(token.lexeme());
        if value.is_some() {
            return value;
        }
        self.parent.as_ref().and_then(|env| env.get(token))
    }

    pub fn assign(&mut self, token: &Token, value: Object) -> Option<&Object> {
        if let Some(object) = self.values.get_mut(token.lexeme()) {
            *object = value;
            return Some(object);
        }
        self.parent
            .as_mut()
            .and_then(|env| env.assign(token, value))
    }
}
