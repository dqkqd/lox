use std::{collections::HashMap, hash::Hash};

use crate::{
    callable::Callable, error::runtime_error::RuntimeError, interpreter::Interpreter,
    object::Object, stmt, token::Token,
};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub(crate) struct LoxClass {
    declaration: stmt::Class,
}

impl LoxClass {
    pub fn new(declaration: stmt::Class) -> Self {
        Self { declaration }
    }

    pub fn new_instance(&mut self, id: usize) -> LoxInstance {
        LoxInstance {
            id,
            lox_class: self.clone(),
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
        _arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError>
    where
        W: std::io::Write,
    {
        let lox_instance = self.new_instance(interpreter.instance_id());
        interpreter.add_new_instance(lox_instance.clone());
        Ok(Object::LoxInstance(lox_instance))
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub(crate) struct LoxInstance {
    id: usize,
    lox_class: LoxClass,
}

impl ToString for LoxInstance {
    fn to_string(&self) -> String {
        format!("<{} instance, id {}>", self.lox_class.name(), self.id)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LoxInstanceFields {
    fields: HashMap<String, Object>,
}

impl LoxInstanceFields {
    pub fn get(&self, name: &Token) -> Option<&Object> {
        self.fields.get(name.lexeme())
    }

    pub fn set(&mut self, name: &Token, value: Object) {
        self.fields.insert(name.lexeme().to_string(), value);
    }
}
