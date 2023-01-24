use crate::{
    environment::EnvironmentTree,
    error::runtime_error::RuntimeError,
    function::{LoxFunction, NativeFunction},
    class::LoxClass,
    interpreter::Interpreter,
    object::Object,
    stmt::{Function, Class}
};

pub(crate) trait Callable {
    fn name(&self) -> &str;
    fn arity(&self) -> usize;
    fn call<W>(
        &mut self,
        interpreter: &mut Interpreter<W>,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError>
    where
        W: std::io::Write;
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) enum LoxCallable {
    LoxFunction(LoxFunction),
    NativeFunction(NativeFunction),
    LoxClass(LoxClass),
}

impl LoxCallable {
    pub fn lox_function(declaration: Function, closure: EnvironmentTree) -> Self {
        LoxCallable::LoxFunction(LoxFunction::new(declaration, closure))
    }

    pub fn native_function(native: NativeFunction) -> Self {
        LoxCallable::NativeFunction(native)
    }

    pub fn lox_class(class: Class) -> Self {
        LoxCallable::LoxClass(LoxClass::new(class))
    }
}

impl ToString for LoxCallable {
    fn to_string(&self) -> String {
        match self {
            LoxCallable::LoxFunction(_) | LoxCallable::NativeFunction(_) => {
                format!("<fn {}>", self.name())
            }
            LoxCallable::LoxClass(_) => format!("<class {}>", self.name())
        }
    }
}

impl Callable for LoxCallable {
    fn name(&self) -> &str {
        match self {
            LoxCallable::LoxFunction(fun) => fun.name(),
            LoxCallable::NativeFunction(fun) => fun.name(),
            LoxCallable::LoxClass(class) => class.name(),
        }
    }

    fn arity(&self) -> usize {
        match self {
            LoxCallable::LoxFunction(fun) => fun.arity(),
            LoxCallable::NativeFunction(fun) => fun.arity(),
            LoxCallable::LoxClass(class) => class.arity(),
        }
    }

    fn call<W>(
        &mut self,
        interpreter: &mut Interpreter<W>,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError>
    where
        W: std::io::Write,
    {
        match self {
            LoxCallable::LoxFunction(fun) => fun.call(interpreter, arguments),
            LoxCallable::NativeFunction(fun) => fun.call(interpreter, arguments),
            LoxCallable::LoxClass(class) => class.call(interpreter, arguments),
        }
    }
}
