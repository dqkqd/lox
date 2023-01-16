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
    errors: Vec<RuntimeError>,
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
            errors: Default::default(),
        }
    }

    pub fn had_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &[RuntimeError] {
        &self.errors
    }

    fn expr(&mut self, e: &Expr) -> InterpreterResult<Object> {
        e.walk_epxr(self)
    }

    fn stmt(&mut self, s: &Stmt) -> InterpreterResult<()> {
        s.walk_stmt(self)
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        self.errors = statements
            .iter()
            .filter_map(|s| match self.stmt(s) {
                Err(error) => Some(error),
                _ => None,
            })
            .collect();
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
            errors: Default::default(),
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

    use std::io::Write;

    use crate::{parser::Parser, scanner::Scanner};

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
        interpreter.interpret(&statements);
        let error_string = interpreter
            .errors()
            .iter()
            .map(|err| err.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        interpreter.write(&error_string)?;

        let result = String::from_utf8(result).unwrap();
        assert_eq!(result.trim(), expected_output.trim());

        Ok(())
    }

    #[test]
    fn only_number_could_be_negation() -> Result<(), std::io::Error> {
        let source = "
-1; 
-nil; 
-true; 
-false; 
-\"a\";
";
        let expected_output = "
[line 3]: RuntimeError: Could not negative non-number
[line 4]: RuntimeError: Could not negative non-number
[line 5]: RuntimeError: Could not negative non-number
[line 6]: RuntimeError: Could not negative non-number
            ";
        test_parser(source, expected_output)
    }

    #[test]
    fn nil_and_false_are_false() -> Result<(), std::io::Error> {
        let source = "
print !1; 
print !nil;
print !true;
print !false;
print !\"a\";
";
        let expected_output = "
false
true
false
true
false
";
        test_parser(source, expected_output)
    }

    #[test]
    fn only_number_can_subtract() -> Result<(), std::io::Error> {
        let source = "
print 1 - 3;
\"a\" - true;
true - nil;
";
        let expected_output = "
-2
[line 3]: RuntimeError: Could not subtract non-number
[line 4]: RuntimeError: Could not subtract non-number
";
        test_parser(source, expected_output)
    }

    #[test]
    fn only_number_can_multiply() -> Result<(), std::io::Error> {
        let source = "
print 5 * 3;
\"a\" * true;
true * nil;
";
        let expected_output = "
15
[line 3]: RuntimeError: Could not multiply non-number
[line 4]: RuntimeError: Could not multiply non-number
";
        test_parser(source, expected_output)
    }

    #[test]
    fn only_number_can_divide() -> Result<(), std::io::Error> {
        let source = "
print 6 / 3;
\"a\" / true;
true / nil;
";
        let expected_output = "
2
[line 3]: RuntimeError: Could not divide non-number
[line 4]: RuntimeError: Could not divide non-number
";
        test_parser(source, expected_output)
    }

    #[test]
    fn divide_by_zero() -> Result<(), std::io::Error> {
        let source = "
1/0;
";
        let expected_output = "
[line 2]: RuntimeError: Division by zero
    ";
        test_parser(source, expected_output)
    }

    #[test]
    fn only_number_or_string_could_add_together() -> Result<(), std::io::Error> {
        let source = "
print 6 + 2;
print \"Hello\" + \" World\";
true + 1;
nil + false;
";
        let expected_output = "
8
Hello World
[line 4]: RuntimeError: Could not add non-number or non-string together
[line 5]: RuntimeError: Could not add non-number or non-string together";
        test_parser(source, expected_output)
    }

    #[test]
    fn only_number_can_be_compare_using_ge_gt_le_lt() -> Result<(), std::io::Error> {
        let source = "
print 1 > 2;
print 1 >= 2;
print 2 < 3;
print 2 <= 2;
true > false;
\"a\" > \"b\";
\"a\" > false;
nil > nil;
";
        let expected_output = "
false
false
true
true
[line 6]: RuntimeError: Could not compare non-number together
[line 7]: RuntimeError: Could not compare non-number together
[line 8]: RuntimeError: Could not compare non-number together
[line 9]: RuntimeError: Could not compare non-number together
";
        test_parser(source, expected_output)
    }

    #[test]
    fn same_kind_object_can_be_true_or_false() -> Result<(), std::io::Error> {
        let source = "
print 1 == 1;
print 1 != 2;
print \"Hello\" == \"Hello\";
print \"Hello\" != \"World\";
print nil == nil;
print true == false;
";
        let expected_output = "
true
true
true
true
true
false
            ";
        test_parser(source, expected_output)
    }

    #[test]
    fn compare_different_kind_object_always_false() -> Result<(), std::io::Error> {
        let source = "
print nil != true;
print 1 == true;
";
        let expected_output = "
true
false
            ";
        test_parser(source, expected_output)
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
    fn nested_block() -> Result<(), std::io::Error> {
        let source = "
var a = \"global a\";
var b = \"global b\";
var c = \"global c\";
{
    var a = \"outer a\";
    var b = \"outer b\";
    {
        var a = \"inner a\";
        print a;
        print b;
        print c;
    }
    print a;
    print b;
    print c;
}
print a;
print b;
print c;
            ";
        let expected_output = "
inner a
outer b
global c
outer a
outer b
global c
global a
global b
global c
";
        test_parser(source, expected_output)
    }

    #[test]
    fn if_then_statement() -> Result<(), std::io::Error> {
        let source = "
if (true) 
    print \"if then\";
";
        let expected_output = "if then";
        test_parser(source, expected_output)
    }

    #[test]
    fn if_then_else_statement() -> Result<(), std::io::Error> {
        let source = "
if (false) 
    print \"if then\";
else
    print \"if then else\";
";
        let expected_output = "if then else";
        test_parser(source, expected_output)
    }

    #[test]
    fn nested_if_then_else() -> Result<(), std::io::Error> {
        let source = "
if (false)
    print \"if then\";
    if (true)
        print \"nested if then\";
    else
        print \"nested if then else\";
";
        let expected_output = "nested if then";
        test_parser(source, expected_output)
    }

    #[test]
    fn if_missing_left_right_paren() -> Result<(), std::io::Error> {
        let source = "
            // missing left paren
            if true;

            // missing right paren
            if (true;

            ";
        let expected_output = "
[line 3]: ParseError: Expected `(`. Found `true`
[line 6]: ParseError: Expected `)`. Found `;`
";
        test_parser(source, expected_output)
    }
}
