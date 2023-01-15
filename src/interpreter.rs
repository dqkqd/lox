use std::io::StdoutLock;

use crate::{
    environment::Environment, error::runtime_error::RuntimeError, expr::Expr, object::Object,
    stmt::Stmt, token::TokenType, visitor::Visitor,
};

pub(crate) struct Interpreter<W>
where
    W: std::io::Write,
{
    writer: W,
    environment: Environment,
}

type InterpreterResult<T> = Result<T, RuntimeError>;

impl<W> Interpreter<W>
where
    W: std::io::Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            environment: Environment::default(),
        }
    }

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

    pub fn write(&mut self, s: &str) -> Result<(), std::io::Error> {
        writeln!(self.writer, "{}", s)
    }
}

impl<'a> Default for Interpreter<StdoutLock<'a>> {
    fn default() -> Self {
        Self {
            writer: std::io::stdout().lock(),
            environment: Environment::default(),
        }
    }
}

impl<W> Visitor<InterpreterResult<Object>, InterpreterResult<()>> for Interpreter<W>
where
    W: std::io::Write,
{
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
                .ok_or_else(|| RuntimeError::undefined_variable(&var.name)),
            Expr::Assign(assign) => {
                let name = &assign.name;
                let value = self.visit_expr(&assign.value)?;
                self.environment
                    .assign(name, value)
                    .cloned()
                    .ok_or_else(|| RuntimeError::undefined_variable(name))
            }
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) -> InterpreterResult<()> {
        match s {
            Stmt::Expression(e) => {
                self.visit_expr(e)?;
            }
            Stmt::Print(e) => {
                let value = self.visit_expr(e)?;
                self.write(&value.to_string())?;
            }
            Stmt::Var(var) => {
                let value = self.visit_expr(&var.expression)?;
                let name = var.identifier.lexeme();
                self.environment.define(name, value);
            }
            Stmt::Block(block) => {
                self.environment.move_to_inner();
                for stmt in &block.statements {
                    self.stmt(stmt)?;
                }
                self.environment.move_to_outer();
            }
            Stmt::If(if_statement) => {
                let condition = self.visit_expr(&if_statement.condition)?;
                if condition.is_truthy() {
                    self.visit_stmt(&if_statement.then_branch)?;
                } else if let Some(else_branch) = &if_statement.else_branch {
                    self.visit_stmt(else_branch)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{error::object_error::ObjectError, parser::Parser, scanner::Scanner};

    use super::*;

    mod test_expr {
        use super::*;
        fn interpret(source: &str) -> InterpreterResult<Object> {
            let mut scanner = Scanner::new(source);
            scanner.scan_tokens();
            assert!(!scanner.had_error());
            let mut parser = Parser::from(&scanner);
            let expr = parser.expression().unwrap();
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

    mod test_stmt {

        use std::io::Write;

        use super::*;

        fn test_parser(source: &str, expected_output: &str) -> Result<(), std::io::Error> {
            let mut result = Vec::new();
            let mut scanner = Scanner::new(source);
            scanner.scan_tokens();
            for error in scanner.errors() {
                writeln!(&mut result, "{:?}", error)?;
            }

            let mut parser = Parser::from(&scanner);
            let statements = parser.parse();
            for error in parser.errors() {
                writeln!(&mut result, "{:?}", error)?;
            }

            let mut interpreter = Interpreter::new(&mut result);
            if let Err(error) = interpreter.interpret(&statements) {
                interpreter.write(&error.to_string())?;
            }

            let result = String::from_utf8(result).unwrap();
            assert_eq!(result.trim(), expected_output.trim());

            Ok(())
        }

        #[test]
        fn variable_declaration() -> Result<(), std::io::Error> {
            let source = "
var x = 1;
print x;
var y
";
            let expected_output = "
[line 5]: ParseError: Expected `;`. Found `EOF`
1";
            test_parser(source, expected_output)?;
            Ok(())
        }

        #[test]
        fn assignment() -> Result<(), std::io::Error> {
            let source = "
var x = 1;
x = 2;
print x;
x = y;
";
            let expected_output = "
2
[line 5]: RuntimeError: Undefined variable `y`";
            test_parser(source, expected_output)?;
            Ok(())
        }

        #[test]
        fn block() -> Result<(), std::io::Error> {
            let source = "
var x = 1;
var y = 2;
{
var x = 3;
print x;
print y;
}
print x;
print y;
";
            let expected_output = "
3
2
1
2
";
            test_parser(source, expected_output)?;
            Ok(())
        }

        #[test]
        fn if_statement() -> Result<(), std::io::Error> {
            let source = "
            // if-then-else
            var x = 1;
            if (x == 1) print \"if then else\";
            else print \"world\";

            // if-then
            x = 2;
            if (x == 2) print \"if then\";

            // nested-if-then-else
            if (true)
                if (false) print \"should not print\";
                else print \"nested if then else\";

            // missing left paren
            if true;

            // missing right paren
            if (true;

            ";
            let expected_output = "
[line 17]: ParseError: Expected `(`. Found `true`
[line 20]: ParseError: Expected `)`. Found `;`
if then else
if then
nested if then else
";
            test_parser(source, expected_output)?;
            Ok(())
        }
    }
}
