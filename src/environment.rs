use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{callable::LoxCallable, function::NativeFunction, object::Object, token::Token};

type EnvironmentLink = Rc<RefCell<EnvironmentNode>>;

#[derive(Debug, Clone, Default, PartialEq)]
struct EnvironmentNode {
    values: HashMap<String, Object>,
    parent: Option<EnvironmentLink>,
}

impl EnvironmentNode {
    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, token: &Token) -> Option<Object> {
        let value = self.values.get(token.lexeme());
        if value.is_some() {
            value.cloned()
        } else {
            self.parent.as_ref().and_then(|env| env.borrow().get(token))
        }
    }

    pub fn assign(&mut self, token: &Token, value: Object) -> Option<Object> {
        if let Some(object) = self.values.get_mut(token.lexeme()) {
            *object = value;
            return Some(object.clone());
        }
        self.parent
            .as_mut()
            .and_then(|env| env.borrow_mut().assign(token, value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EnvironmentTree {
    env: Option<EnvironmentLink>,
    global: EnvironmentLink,
}

impl Default for EnvironmentTree {
    fn default() -> Self {
        Self {
            env: None,
            global: EnvironmentLink::default(),
        }
        .with_prelude()
    }
}

impl EnvironmentTree {
    fn with_prelude(self) -> Self {
        // define global environment
        let clock = NativeFunction::clock();
        self.global.borrow_mut().define(
            "clock",
            Object::Callable(LoxCallable::native_function(clock)),
        );
        self
    }

    pub fn define(&mut self, name: &str, value: Object) {
        let env = match self.env.as_ref() {
            Some(env) => env,
            None => self.global.as_ref(),
        };
        env.borrow_mut().define(name, value);
    }

    pub fn get(&self, token: &Token) -> Option<Object> {
        let result = self.env.as_ref().and_then(|env| env.borrow().get(token));
        if result.is_some() {
            result
        } else {
            self.global.borrow().get(token)
        }
    }

    pub fn assign(&mut self, token: &Token, value: Object) -> Option<Object> {
        let result = self
            .env
            .as_ref()
            .and_then(|env| env.borrow_mut().assign(token, value.clone()));
        if result.is_some() {
            result
        } else {
            self.global.borrow_mut().assign(token, value)
        }
    }

    pub fn append(&self) -> Self {
        EnvironmentTree {
            env: Some(Rc::new(RefCell::new(EnvironmentNode {
                parent: self.env.clone(),
                ..Default::default()
            }))),
            global: Rc::clone(&self.global),
        }
    }

    pub fn pop(&self) -> Self {
        let parent = self
            .env
            .as_ref()
            .and_then(|env| env.borrow().parent.clone());
        Self {
            env: parent,
            global: Rc::clone(&self.global),
        }
    }

    pub fn move_to_inner(&mut self) {
        *self = self.append();
    }

    pub fn move_to_outer(&mut self) {
        *self = self.pop();
    }
}
