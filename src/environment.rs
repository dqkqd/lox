use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{object::Object, token::Token};

type EnvironmentLink = Rc<RefCell<EnvironmentNode>>;

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
pub(crate) struct EnvironmentTree {
    // root is actually a child
    root: EnvironmentLink,
}

impl EnvironmentTree {
    pub fn define(&mut self, name: &str, value: Object) {
        self.root.borrow_mut().define(name, value);
    }

    pub fn get(&self, token: &Token) -> Option<Object> {
        self.root.borrow().get(token)
    }

    pub fn assign(&mut self, token: &Token, value: Object) -> Option<Object> {
        self.root.borrow_mut().assign(token, value)
    }

    fn move_to_child(&self) -> Self {
        EnvironmentTree {
            root: Rc::new(RefCell::new(EnvironmentNode {
                parent: Some(self.root.clone()),
                ..Default::default()
            })),
        }
    }

    fn move_to_parent(&self) -> Option<Self> {
        self.root.borrow().parent.clone().map(|root| Self { root })
    }

    pub fn move_to_inner(&mut self) {
        *self = self.move_to_child();
    }

    pub fn move_to_outer(&mut self) {
        *self = self.move_to_parent().unwrap_or_default();
    }
}
