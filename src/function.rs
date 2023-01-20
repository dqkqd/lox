use crate::{
    callable::LoxCallable, error::runtime_error::RuntimeError, interpreter::Interpreter,
    object::Object, stmt::Function, token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LoxFunction {
    declaration: Function,
}

impl LoxFunction {
    pub fn new(declaration: Function) -> Self {
        Self { declaration }
    }

    pub fn name(&self) -> &Token {
        &self.declaration.name
    }
}

impl LoxCallable for LoxFunction {
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
        return Ok(Object::Null);
    }
}

        if self.arity() != arguments.len() {
            return Err(RuntimeError::number_arguments_mismatch(
                self.declaration.name.line(),
                self.arity(),
                arguments.len(),
            ));
        }

        for (param, arg) in self.declaration.params.iter().zip(arguments) {
            interpreter.environment_mut().define(param.lexeme(), arg);
        }
        interpreter.stmt(&self.declaration.body)?;
        interpreter.environment_mut().move_to_outer();
        return Ok(Object::Null);
    }
}
