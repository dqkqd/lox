use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use crate::{
    callable::{Callable, LoxCallable},
    error::runtime_error::RuntimeError,
    function::LoxFunction,
    interpreter::Interpreter,
    object::Object,
    stmt,
    token::Token,
};

#[derive(Debug, Clone)]
pub(crate) struct LoxClass {
    declaration: stmt::Class,
    methods: HashMap<String, LoxFunction>,
}

impl PartialEq for LoxClass {
    fn eq(&self, other: &Self) -> bool {
        self.declaration == other.declaration
    }
}

impl Eq for LoxClass {}

impl Hash for LoxClass {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.declaration.hash(state)
    }
}

impl LoxClass {
    pub fn new(declaration: stmt::Class, methods: HashMap<String, LoxFunction>) -> Self {
        Self {
            declaration,
            methods,
        }
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

impl LoxInstance {
    pub fn find_method(&self, name: &str) -> Option<&LoxFunction> {
        self.lox_class.methods.get(name)
    }
}

impl ToString for LoxInstance {
    fn to_string(&self) -> String {
        format!("<{} instance, id {}>", self.lox_class.name(), self.id)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LoxInstanceFields {
    instance: LoxInstance,
    fields: HashMap<String, Object>,
}

impl From<LoxInstance> for LoxInstanceFields {
    fn from(instance: LoxInstance) -> Self {
        LoxInstanceFields {
            instance,
            fields: Default::default(),
        }
    }
}
impl LoxInstanceFields {
    pub fn get(&self, name: &Token) -> Option<Object> {
        let object = self.fields.get(name.lexeme());
        if object.is_some() {
            return object.cloned();
        }
        self.instance
            .find_method(name.lexeme())
            .map(|fun| Object::Callable(LoxCallable::LoxFunction(fun.clone())))
    }

    pub fn set(&mut self, name: &Token, value: Object) {
        self.fields.insert(name.lexeme().to_string(), value);
    }
}
