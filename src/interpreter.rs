use crate::{
    error::runtime_error::RuntimeError, expr::Expr, object::Object, token::TokenType,
    visitor::Visitor,
};

#[derive(Default)]
pub(crate) struct Interpreter;

type InterpreterResult = Result<Object, RuntimeError>;

impl Interpreter {
    pub fn expr(&mut self, e: &Expr) -> InterpreterResult {
        e.walk_epxr(self)
    }
}

impl Visitor<InterpreterResult> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> InterpreterResult {
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
                        Ok((lhs.ge(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::GreaterEqual => {
                        Ok((lhs.gt(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::Less => {
                        Ok((lhs.le(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
                    }
                    TokenType::LessEqual => {
                        Ok((lhs.lt(&rhs)).map_err(|err| RuntimeError::from((line, err)))?)
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
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{error::object_error::ObjectError, parser::Parser, scanner::Scanner};

    use super::*;

    fn interpret(source: &str) -> InterpreterResult {
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();
        assert!(!scanner.had_error());
        let mut parser = Parser::from(&scanner);
        let expr = parser.expression().unwrap();
        dbg!(&expr);

        let mut interpreter = Interpreter::default();
        interpreter.expr(&expr)
    }

    fn test_interpreter(sources: &[&str], expected_results: &[InterpreterResult]) {
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
