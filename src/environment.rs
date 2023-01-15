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

    pub fn move_to_inner(&mut self) {
        // move current to fresh environment which's parent is the old one
        let mut outer_environment = Environment::default();
        std::mem::swap(self, &mut outer_environment);
        self.parent = Some(Box::new(outer_environment));
    }

    pub fn move_to_outer(&mut self) {
        // discard current environment and move to parent
        let outer = self.parent.take();
        if let Some(mut outer) = outer {
            std::mem::swap(self, &mut outer)
        }
    }
}
