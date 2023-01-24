use std::collections::HashMap;

use crate::{
    error::{resolve_error::ResolveError, ErrorReporter, reporter::TestErrorReporter},
    expr::Expr,
    interpreter::Interpreter,
    stmt::Stmt,
    token::Token,
    visitor::Visitor,
};

pub(crate) struct Resolver<'a, W>
where
    W: std::io::Write,
{
    scopes: Vec<HashMap<String, bool>>,
    errors: Vec<ResolveError>,
    interpreter: &'a mut Interpreter<W>,
    function_level: usize,
}

type ResolveResult<T> = Result<T, ResolveError>;

impl<'a, W> ErrorReporter<ResolveError> for Resolver<'a, W>
where
    W: std::io::Write,
{
    fn errors(&self) -> &[ResolveError] {
        &self.errors
    }
}

impl<'a, W> Resolver<'a, W>
where
    W: std::io::Write,
{
    pub fn new(interpreter: &'a mut Interpreter<W>) -> Self {
        Self {
            interpreter,
            errors: Default::default(),
            scopes: Default::default(),
            function_level: 0,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Default::default());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, ident: &Token) -> ResolveResult<()> {
        if let Some(last) = self.scopes.last_mut() {
            if last.contains_key(ident.lexeme()) {
                return Err(ResolveError::already_declared(ident));
            }
            last.insert(ident.lexeme().to_string(), false);
        }
        Ok(())
    }

    fn define(&mut self, ident: &Token) {
        if let Some(last) = self.scopes.last_mut() {
            if let Some(value) = last.get_mut(ident.lexeme()) {
                *value = true;
            }
        }
    }

    fn resolve_local(&mut self, expr: Expr, name: &Token) {
        if let Some((depth, _)) = self
            .scopes
            .iter()
            .rev()
            .enumerate()
            .find(|(_, scope)| scope.contains_key(name.lexeme()))
        {
            self.interpreter.resolve(expr, depth)
        }
    }

    pub fn resolve(&mut self, statements: &[Stmt]) {
        self.errors = statements
            .iter()
            .filter_map(|s| match self.visit_stmt(s) {
                Err(error) => Some(error),
                _ => None,
            })
            .collect();
    }
}

impl<'a, W> Visitor<ResolveResult<()>, ResolveResult<()>> for Resolver<'a, W>
where
    W: std::io::Write,
{
    fn visit_expr(&mut self, e: &Expr) -> ResolveResult<()> {
        match e {
            Expr::Binary(binary) => {
                self.visit_expr(&binary.left)?;
                self.visit_expr(&binary.right)?;
            }
            Expr::Unary(unary) => {
                self.visit_expr(&unary.right)?;
            }
            Expr::Literal(_) => (),
            Expr::Grouping(group) => {
                self.visit_expr(&group.expr)?;
            }
            Expr::Variable(var) => {
                if self
                    .scopes
                    .last()
                    .and_then(|scope| scope.get(var.name.lexeme()))
                    == Some(&false)
                {
                    return Err(ResolveError::read_during_initializer(&var.name));
                }
                // todo: move instead of clone
                self.resolve_local(e.clone(), &var.name);
            }
            Expr::Assign(assign) => {
                self.visit_expr(&assign.value)?;
                // todo: move instead of clone
                self.resolve_local(e.clone(), &assign.name);
            }
            Expr::Logical(logical) => {
                self.visit_expr(&logical.left)?;
                self.visit_expr(&logical.right)?;
            }
            Expr::Call(call) => {
                self.visit_expr(&call.callee)?;
                for arg in &call.arguments {
                    self.visit_expr(arg)?;
                }
            }
        }
        Ok(())
    }

    fn visit_stmt(&mut self, s: &Stmt) -> ResolveResult<()> {
        match s {
            Stmt::Expression(expr) => {
                self.visit_expr(expr)?;
            }
            Stmt::Print(p) => {
                self.visit_expr(p)?;
            }
            Stmt::Return(r) => {
                if self.function_level == 0 {
                    return Err(ResolveError::return_from_top_level(&r.keyword));
                }
                self.visit_expr(&r.value)?;
            }
            Stmt::Function(fun) => {
                self.declare(&fun.name)?;
                self.define(&fun.name);
                self.begin_scope();
                self.function_level += 1;
                for param in &fun.params {
                    self.declare(param)?;
                    self.define(param);
                }
                let result = self.visit_stmt(&fun.body);
                self.function_level -= 1;
                self.end_scope();
                result?;
            }
            Stmt::Var(var) => {
                self.declare(&var.identifier)?;
                self.visit_expr(&var.expression)?;
                self.define(&var.identifier);
            }
            Stmt::Block(block) => {
                self.begin_scope();
                let error = block
                    .statements
                    .iter()
                    .map(|s| self.visit_stmt(s))
                    .find(|r| r.is_err());
                self.end_scope();
                if let Some(error) = error {
                    return error;
                };
            }
            Stmt::If(i) => {
                self.visit_expr(&i.condition)?;
                self.visit_stmt(&i.then_branch)?;
                if let Some(else_branch) = &i.else_branch {
                    self.visit_stmt(else_branch)?;
                }
            }
            Stmt::While(w) => {
                self.visit_expr(&w.condition)?;
                self.visit_stmt(&w.body)?;
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{ast_repr::AstRepr, parser::Parser, scanner::Scanner};

    use super::*;

    fn test_resolver(source: &str, expected_output: &str) -> Result<(), std::io::Error> {
        let mut result = Vec::new();
        let mut interpreter = Interpreter::new(&mut result);

        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();
        interpreter.write(&scanner.error_string())?;

        let mut parser = Parser::from(&scanner);
        let statements = parser.parse();
        interpreter.write(&parser.error_string())?;

        let mut resolver = Resolver::new(&mut interpreter);
        resolver.resolve(&statements);
        let error_string = resolver.error_string();
        interpreter.write(&error_string)?;

        let mut ast_repr = AstRepr::default();
        let expr_depth_string = interpreter
            .locals()
            .iter()
            .map(|(expr, depth)| format!("{}: {}", ast_repr.expr(expr), depth))
            .collect::<Vec<_>>()
            .join("\n");
        interpreter.write(&expr_depth_string)?;

        let result = String::from_utf8(result).unwrap();
        assert_eq!(result.trim(), expected_output.trim());

        Ok(())
    }

    #[test]
    fn declare_using_its_own_initializer() -> Result<(), std::io::Error> {
        let source = "
var a = 2;
{
    var a = a;
}
        ";

        let expected_output = "
[line 4]: ResolveError: Couldn't read `a` in its own initializer
        ";

        test_resolver(source, expected_output)
    }

    #[test]
    fn already_declared_variable_in_scope() -> Result<(), std::io::Error> {
        let source = "
fun bad() {
    var a = 1;
    var a = 2;
}        
        ";

        let expected_output = "
[line 4]: ResolveError: Already a variable `a` in this scope.
        ";

        test_resolver(source, expected_output)
    }

    #[test]
    fn return_at_top_level() -> Result<(), std::io::Error> {
        let source = "
fun x() {
    return 1;
}

{
    return 2;
}

return 1;
";

        let expected_output = "
[line 7]: ResolveError: Could not return from top level code
[line 10]: ResolveError: Could not return from top level code
";

        test_resolver(source, expected_output)
    }
}
