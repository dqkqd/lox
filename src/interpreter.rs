use crate::{
    environment::Environment, error::runtime_error::RuntimeError, expr::Expr, object::Object,
    stmt::Stmt, token::TokenType, visitor::Visitor,
};

#[derive(Default)]
pub(crate) struct Interpreter {
    environment: Environment,
}

type InterpreterResult<T> = Result<T, RuntimeError>;

impl Interpreter {
    fn expr(&mut self, e: &Expr) -> InterpreterResult<Object> {
        e.walk_epxr(self)
    }

    fn stmt(&mut self, s: &Stmt) -> InterpreterResult<()> {
        s.walk_stmt(self)
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> InterpreterResult<()> {
        for stmt in statements {
            self.stmt(stmt)?;
        }
        Ok(())
    }
}

impl Visitor<InterpreterResult<Object>, InterpreterResult<()>> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> InterpreterResult<Object> {
        match e {
            Expr::Binary(binary) => {
                let lhs = self.visit_expr(&binary.left)?;
                let rhs = self.visit_expr(&binary.right)?;
                let line = binary.operator.line();
                match binary.operator.token_type() {
                    TokenType::Minus => {
                        Ok((lhs - rhs).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::Star => {
                        Ok((lhs * rhs).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::Slash => {
                        Ok((lhs / rhs).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::Plus => {
                        Ok((lhs + rhs).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::Greater => {
                        Ok((lhs.gt(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::GreaterEqual => {
                        Ok((lhs.ge(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::Less => {
                        Ok((lhs.lt(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::LessEqual => {
                        Ok((lhs.le(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::BangEqual => {
                        Ok((lhs.ne(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::EqualEqual => {
                        Ok((lhs.eq(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    _ => unimplemented!(),
                }
            }
            Expr::Unary(unary) => {
                let rhs = self.visit_expr(&unary.right)?;
                let line = unary.operator.line();
                match unary.operator.token_type() {
                    TokenType::Minus => Ok((-rhs).map_err(|err| RuntimeError::from((line, err)))?),
                    TokenType::Bang => Ok(Object::Bool(!rhs.is_truthy())),
                    _ => unimplemented!(),
                }
            }
            Expr::Literal(object) => Ok(object.clone()),
            Expr::Grouping(group) => Ok(self.visit_expr(&group.expr)?),
            Expr::Variable(var) => self
                .environment
                .get(&var.name)
                .cloned()
                .ok_or_else(|| RuntimeError::undefined_variable(var.name.line(), &var.name)),
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) -> InterpreterResult<()> {
        match s {
            Stmt::Expression(e) => {
                self.visit_expr(e)?;
            }
            Stmt::Print(e) => {
                let value = self.visit_expr(e)?;
                // @todo, print to specific output stream
                println!("{}", value.to_string());
            }
            Stmt::Var(var) => {
                let value = self.visit_expr(&var.expression)?;
                let name = var.identifier.lexeme();
                self.environment.define(name, value);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{error::object_error::ObjectError, parser::Parser, scanner::Scanner};

    use super::*;

    fn interpret(source: &str) -> InterpreterResult<Object> {
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();
        assert!(!scanner.had_error());
        let mut parser = Parser::from(&scanner);
        let expr = parser.expression().unwrap();
        dbg!(&expr);

        let mut interpreter = Interpreter::default();
        interpreter.expr(&expr)
    }

    fn test_interpreter(sources: &[&str], expected_results: &[InterpreterResult<Object>]) {
        for (&src, expected) in std::iter::zip(sources, expected_results) {
            let result = interpret(src);
            assert_eq!(&result, expected);
        }
    }

    #[test]
    fn expr_only_number_could_be_unary() {
        let sources = ["-1", "-nil", "-true", "-false", "-\"a\""];
        let expected_results = [
            Ok(Object::Number(-1.0)),
            Err(RuntimeError::from((1, ObjectError::negative()))),
            Err(RuntimeError::from((1, ObjectError::negative()))),
            Err(RuntimeError::from((1, ObjectError::negative()))),
            Err(RuntimeError::from((1, ObjectError::negative()))),
        ];
        test_interpreter(&sources, &expected_results);
    }

    #[test]
    fn expr_nil_and_false_are_false() {
        let sources = ["!1", "!nil", "!true", "!false", "!\"a\""];
        let expected_results = [
            Ok(Object::Bool(false)),
            Ok(Object::Bool(true)),
            Ok(Object::Bool(false)),
            Ok(Object::Bool(true)),
            Ok(Object::Bool(false)),
        ];
        test_interpreter(&sources, &expected_results);
    }

    #[test]
    fn expr_binary_subtract() {
        let sources = ["2-3", "\"a\" - true"];
        let expected_results = [
            Ok(Object::Number(-1.0)),
            Err(RuntimeError::from((1, ObjectError::subtract()))),
        ];
        test_interpreter(&sources, &expected_results);
    }

    #[test]
    fn expr_binary_multiplication() {
        let sources = ["2 * 3", "nil * nil"];
        let expected_results = [
            Ok(Object::Number(6.0)),
            Err(RuntimeError::from((1, ObjectError::multiplication()))),
        ];
        test_interpreter(&sources, &expected_results);
    }

    #[test]
    fn expr_binary_division() {
        let sources = ["3 / 2", "nil / nil"];
        let expected_results = [
            Ok(Object::Number(1.5)),
            Err(RuntimeError::from((1, ObjectError::division()))),
        ];
        test_interpreter(&sources, &expected_results);
    }

    #[test]
    fn expr_binary_add() {
        let sources = ["3 + 2", "\"Hello\" + \" World\"", "true + 1", "nil + 2"];
        let expected_results = [
            Ok(Object::Number(5.0)),
            Ok(Object::String("Hello World".to_string())),
            Err(RuntimeError::from((1, ObjectError::addition()))),
            Err(RuntimeError::from((1, ObjectError::addition()))),
        ];
        test_interpreter(&sources, &expected_results);
    }

    #[test]
    fn expr_binary_comparision() {
        let sources = [
            "1 > 2",
            "1 >= 2",
            "1 < 2",
            "1 <= 2",
            "true > false",
            "\"a\" >= \"b\"",
            "nil < nil",
            "nil <= true",
            "true == true",
            "true == false",
            "true != true",
            "true != false",
            "nil == nil",
            "nil == 1",
            "nil != nil",
            "nil != 1",
            "1 == 2",
            "1 == 1",
            "1 != 2",
            "1 != 1",
            "\"hello\" == \"hello\"",
            "\"hello\" == \"world\"",
            "\"hello\" != \"hello\"",
            "\"hello\" != \"world\"",
        ];
        let expected_results = [
            Ok(Object::Bool(false)),
            Ok(Object::Bool(false)),
            Ok(Object::Bool(true)),
            Ok(Object::Bool(true)),
            Err(RuntimeError::from((1, ObjectError::comparision()))),
            Err(RuntimeError::from((1, ObjectError::comparision()))),
            Err(RuntimeError::from((1, ObjectError::comparision()))),
            Err(RuntimeError::from((1, ObjectError::comparision()))),
            Ok(Object::Bool(true)),
            Ok(Object::Bool(false)),
            Ok(Object::Bool(false)),
            Ok(Object::Bool(true)),
            Ok(Object::Bool(true)),
            Ok(Object::Bool(false)),
            Ok(Object::Bool(false)),
            Ok(Object::Bool(true)),
            Ok(Object::Bool(false)),
            Ok(Object::Bool(true)),
            Ok(Object::Bool(true)),
            Ok(Object::Bool(false)),
            Ok(Object::Bool(true)),
            Ok(Object::Bool(false)),
            Ok(Object::Bool(false)),
            Ok(Object::Bool(true)),
        ];
        test_interpreter(&sources, &expected_results);
    }
}
