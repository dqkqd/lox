use crate::{
    callable::Callable,
    interpreter::Interpreter,
    object::Object,
    error::runtime_error::RuntimeError,
    stmt};

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) struct LoxClass {
    declaration: stmt::Class
}

impl LoxClass {
    pub fn new(declaration: stmt::Class) -> Self {
        Self {
            declaration
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
         let lox_instance = LoxInstance::new(self);
         Ok(Object::LoxInstance(lox_instance))
     }
}


#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) struct LoxInstance {
    lox_class: LoxClass,
}

impl ToString for LoxInstance {
    fn to_string(&self) -> String {
        format!("<{} instance>", self.lox_class.name())
    }
}

impl LoxInstance {
    pub fn new(lox_class: &LoxClass) -> Self {
        Self {
            lox_class: lox_class.clone()
        }
    }
}

