use std::time::SystemTime;

use crate::{
    callable::Callable, error::runtime_error::RuntimeError, interpreter::Interpreter,
    object::Object, stmt::Function,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LoxFunction {
    declaration: Function,
}

impl LoxFunction {
    pub fn new(declaration: Function) -> Self {
        Self { declaration }
    }
}

impl Callable for LoxFunction {
    fn name(&self) -> &str {
        self.declaration.name.lexeme()
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call<W>(
        &self,
        interpreter: &mut Interpreter<W>,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError>
    where
        W: std::io::Write,
    {
        interpreter.environment_mut().move_to_inner();
        for (param, arg) in self.declaration.params.iter().zip(arguments) {
            interpreter.environment_mut().define(param.lexeme(), arg);
        }
        interpreter.stmt(&self.declaration.body)?;
        interpreter.environment_mut().move_to_outer();
        Ok(Object::Null)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NativeFunction {
    Clock(Clock),
}

impl NativeFunction {
    pub fn clock() -> Self {
        NativeFunction::Clock(Clock::default())
    }
}

impl Callable for NativeFunction {
    fn name(&self) -> &str {
        match self {
            NativeFunction::Clock(clock) => clock.name(),
        }
    }

    fn arity(&self) -> usize {
        match self {
            NativeFunction::Clock(clock) => clock.arity(),
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
            NativeFunction::Clock(clock) => clock.call(interpreter, arguments),
        }
    }
}

// native clock function
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Clock;

impl Callable for Clock {
    fn name(&self) -> &str {
        "clock"
    }

    fn arity(&self) -> usize {
        0
    }

    fn call<W>(&self, _: &mut Interpreter<W>, _: Vec<Object>) -> Result<Object, RuntimeError>
    where
        W: std::io::Write,
    {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        Ok(Object::Number(now.as_millis() as f64))
    }
}
