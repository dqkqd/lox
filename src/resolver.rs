use std::collections::HashMap;

use crate::{
    error::resolve_error::ResolveError, expr::Expr, interpreter::Interpreter, stmt::Stmt,
    token::Token, visitor::Visitor,
};

pub(crate) struct Resolver<W>
where
    W: std::io::Write,
{
    scopes: Vec<HashMap<String, bool>>,
    interpreter: Interpreter<W>,
}

type ResolveResult<T> = Result<T, ResolveError>;

impl<W> Resolver<W>
where
    W: std::io::Write,
{
    fn begin_scope(&mut self) {
        self.scopes.push(Default::default());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, ident: &Token) {
        if let Some(last) = self.scopes.last_mut() {
            last.insert(ident.lexeme().to_string(), false);
        }
    }

    fn define(&mut self, ident: &Token) {
        if let Some(last) = self.scopes.last_mut() {
            if let Some(value) = last.get_mut(ident.lexeme()) {
                *value = true;
            }
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        self.scopes
            .iter()
            .rev()
            .enumerate()
            .find(|(_, scope)| scope.contains_key(name.lexeme()))
            .map(|o| {
                /* */
                todo!("resolve by interpreter")
            });
    }
}

impl<W> Visitor<ResolveResult<()>, ResolveResult<()>> for Resolver<W>
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
                let resolved = self
                    .scopes
                    .last()
                    .and_then(|scope| scope.get(var.name.lexeme()))
                    .unwrap_or(&false);
                if resolved == &false {
                    return Err(ResolveError::read_during_initializer(&var.name));
                }
                self.resolve_local(e, &var.name);
            }
            Expr::Assign(assign) => {
                self.visit_expr(&assign.value)?;
                self.resolve_local(e, &assign.name);
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
                self.visit_expr(&r.value)?;
            }
            Stmt::Function(fun) => {
                self.declare(&fun.name);
                self.define(&fun.name);
                self.begin_scope();
                for param in &fun.params {
                    self.declare(param);
                    self.define(param);
                }
                let result = self.visit_stmt(&fun.body);
                self.end_scope();
                result?;
            }
            Stmt::Var(var) => {
                self.declare(&var.identifier);
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
