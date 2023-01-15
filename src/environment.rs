use std::collections::HashMap;

use crate::{object::Object, token::Token};

#[derive(Debug, Default)]
pub(crate) struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&mut self, token: &Token) -> Option<&Object> {
        self.values.get(token.lexeme())
    }
}
