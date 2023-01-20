use crate::{
    error::runtime_error::RuntimeError,
    function::{LoxFunction, NativeFunction},
    interpreter::Interpreter,
    object::Object,
    stmt::Function,
};

pub(crate) trait Callable {
    fn name(&self) -> &str;
    fn arity(&self) -> usize;
    fn call<W>(
        &self,
        interpreter: &mut Interpreter<W>,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError>
    where
        W: std::io::Write;
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LoxCallable {
    LoxFunction(LoxFunction),
    NativeFunction(NativeFunction),
}

impl LoxCallable {
    pub fn lox_function(declaration: Function) -> Self {
        LoxCallable::LoxFunction(LoxFunction::new(declaration))
    }

    pub fn native_function(native: NativeFunction) -> Self {
        LoxCallable::NativeFunction(native)
    }
}

impl Callable for LoxCallable {
    fn name(&self) -> &str {
        match self {
            LoxCallable::LoxFunction(fun) => fun.name(),
            LoxCallable::NativeFunction(fun) => fun.name(),
        }
    }

    fn arity(&self) -> usize {
        match self {
            LoxCallable::LoxFunction(fun) => fun.arity(),
            LoxCallable::NativeFunction(fun) => fun.arity(),
        }
    }

    fn call<W>(
        &self,
        interpreter: &mut Interpreter<W>,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError>
    where
        W: std::io::Write,
    {
        match self {
            LoxCallable::LoxFunction(fun) => fun.call(interpreter, arguments),
            LoxCallable::NativeFunction(fun) => fun.call(interpreter, arguments),
        }
    }
}
