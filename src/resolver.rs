use std::collections::HashMap;

use crate::{
    error::{reporter::ErrorReporter, resolve_error::ResolveError},
    expr::Expr,
    interpreter::Interpreter,
    object::Object,
    stmt::Stmt,
    token::Token,
    visitor::Visitor,
};

#[derive(Debug, Clone, Copy)]
enum FunctionType {
    Null,
    Function,
    Method,
    Initializer,
}

impl FunctionType {
    fn next_level(&mut self, fun_name: &str) {
        let function_type = match &self {
            FunctionType::Null => FunctionType::Function,
            FunctionType::Function => FunctionType::Function,
            FunctionType::Method => {
                if fun_name == "init" {
                    FunctionType::Initializer
                } else {
                    FunctionType::Method
                }
            }
            FunctionType::Initializer => FunctionType::Function,
        };
        *self = function_type;
    }
}
pub(crate) struct Resolver<'a, W>
where
    W: std::io::Write,
{
    scopes: Vec<HashMap<String, bool>>,
    errors: Vec<ResolveError>,
    interpreter: &'a mut Interpreter<W>,
    function_type: FunctionType,
    class_level: usize,
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
            function_type: FunctionType::Null,
            class_level: 0,
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
            Expr::Get(get) => self.visit_expr(&get.object)?,
            Expr::Set(set) => {
                self.visit_expr(&set.value)?;
                self.visit_expr(&set.object)?;
            }
            Expr::This(this) => {
                if self.class_level == 0 {
                    return Err(ResolveError::call_this_outside_class(&this.keyword));
                }
                self.resolve_local(e.clone(), &this.keyword);
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
                match self.function_type {
                    FunctionType::Null => {
                        return Err(ResolveError::return_from_top_level(&r.keyword))
                    }
                    FunctionType::Initializer => {
                        if &r.value != &Expr::Literal(Object::Null) {
                            return Err(ResolveError::return_inside_init(&r.keyword));
                        }
                    }
                    _ => (),
                };
                self.visit_expr(&r.value)?;
            }
            Stmt::Function(fun) => {
                self.declare(&fun.name)?;
                self.define(&fun.name);
                self.begin_scope();

                let old_function_type = self.function_type;
                self.function_type.next_level(fun.name.lexeme());

                for param in &fun.params {
                    self.declare(param)?;
                    self.define(param);
                }
                let result = self.visit_stmt(&fun.body);

                self.function_type = old_function_type;

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
            Stmt::Class(class) => {
                self.declare(&class.name)?;
                self.define(&class.name);

                self.class_level += 1;
                self.begin_scope();
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert("this".to_string(), false);

                let old_function_type = self.function_type;

                for method in &class.methods {
                    self.function_type = FunctionType::Method;
                    self.visit_stmt(method)?;
                }

                self.function_type = old_function_type;
                self.end_scope();
                self.class_level -= 1;
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{error::reporter::Reporter, parser::Parser, scanner::Scanner, source::SourcePos};

    use super::*;

    fn test_resolver(source: &str, expected_output: &str) -> Result<(), std::io::Error> {
        let source_pos = SourcePos::new(source);
        let reporter = Reporter::new(&source_pos);

        let mut result = Vec::new();
        let mut interpreter = Interpreter::new(&mut result);

        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();
        interpreter.write(&scanner.error_msg(&reporter))?;

        let mut parser = Parser::from(&scanner);
        let statements = parser.parse();
        interpreter.write(&parser.error_msg(&reporter))?;

        let mut resolver = Resolver::new(&mut interpreter);
        resolver.resolve(&statements);
        let error_msg = resolver.error_msg(&reporter);
        interpreter.write(&error_msg)?;

        let result = String::from_utf8(result).unwrap();
        assert_eq!(result.trim(), expected_output.trim());

        Ok(())
    }

    #[test]
    fn declare_using_its_own_initializer() -> Result<(), std::io::Error> {
        let source = r#"
var a = 2;
{
    var a = a;
}
"#;

        let expected_output = r#"
[line 4]: ResolveError: Couldn't read `a` in its own initializer
    var a = a;
            ^
"#;

        test_resolver(source, expected_output)
    }

    #[test]
    fn already_declared_variable_in_scope() -> Result<(), std::io::Error> {
        let source = r#"
fun bad() {
    var a = 1;
    var a = 2;
}        
"#;

        let expected_output = r#"
[line 4]: ResolveError: Already a variable `a` in this scope.
    var a = 2;
        ^
"#;

        test_resolver(source, expected_output)
    }

    #[test]
    fn return_at_top_level() -> Result<(), std::io::Error> {
        let source = r#"
fun x() {
    return 1;
}

{
    return 2;
}

return 1;
"#;

        let expected_output = r#"
[line 7]: ResolveError: Could not return from top level code
    return 2;
    ^^^^^^
[line 10]: ResolveError: Could not return from top level code
return 1;
^^^^^^
"#;

        test_resolver(source, expected_output)
    }

    #[test]
    fn call_this_outside_class() -> Result<(), std::io::Error> {
        let source = r#"
print this;
class Hello {}
fun f() {
    return this;
}
"#;

        let expected_output = r#"
[line 2]: ResolveError: Could not use `this` outside of a class
print this;
      ^^^^
[line 5]: ResolveError: Could not use `this` outside of a class
    return this;
           ^^^^
"#;

        test_resolver(source, expected_output)
    }

    #[test]
    fn dont_allow_return_inside_init() -> Result<(), std::io::Error> {
        let source = r#"
class Hello {
    init() {
        return "something else";
    }
}
"#;

        let expected_output = r#"
[line 4]: ResolveError: Could not return inside constructor
        return "something else";
        ^^^^^^
"#;

        test_resolver(source, expected_output)
    }
}
