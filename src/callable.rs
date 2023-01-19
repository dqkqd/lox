use crate::{error::runtime_error::RuntimeError, interpreter::Interpreter, object::Object};

pub(crate) trait LoxCallable {
    fn arity(&self) -> usize;
    fn call<W>(
        &self,
        interpreter: &mut Interpreter<W>,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError>
    where
        W: std::io::Write;
}
