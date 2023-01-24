use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::{
    callable::Callable, error::runtime_error::RuntimeError, interpreter::Interpreter,
    object::Object, stmt, token::Token,
};

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) struct LoxClass {
    declaration: stmt::Class,
}

impl LoxClass {
    pub fn new(declaration: stmt::Class) -> Self {
        Self { declaration }
    }

    pub fn new_instance(&self) -> LoxInstance {
        LoxInstance {
            lox_class: self.clone(),
            fields: Default::default(),
        }
    }
}

impl Callable for LoxClass {
    fn name(&self) -> &str {
        self.declaration.name.lexeme()
    }

    fn arity(&self) -> usize {
        0
    }

    fn call<W>(
        &mut self,
        interpreter: &mut Interpreter<W>,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError>
    where
        W: std::io::Write,
    {
        let lox_instance = self.new_instance();
        Ok(Object::LoxInstance(lox_instance))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LoxInstance {
    lox_class: LoxClass,
    fields: HashMap<String, Object>,
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for LoxInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // @todo! should we hash fields as well?
        self.lox_class.hash(state);
    }
}

impl ToString for LoxInstance {
    fn to_string(&self) -> String {
        format!("<{} instance>", self.lox_class.name())
    }
}

impl LoxInstance {
    pub fn get(&self, name: &Token) -> Option<&Object> {
        self.fields.get(name.lexeme())
    }

    pub fn set(&mut self, name: &Token, value: Object) {
        // @todo: check instance have field name beforehand
        self.fields.insert(name.lexeme().to_string(), value);
    }
}
