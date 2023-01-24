use crate::{expr::Expr, object::Object, stmt::Stmt, visitor::Visitor};

#[derive(Default)]
pub(crate) struct AstRepr;

#[allow(dead_code)]
impl AstRepr {
    pub fn expr(&mut self, e: &Expr) -> String {
        e.walk_epxr(self)
    }

    pub fn stmt(&mut self, s: &Stmt) -> String {
        s.walk_stmt(self)
    }

    pub fn repr(&mut self, statements: &[Stmt]) -> String {
        statements
            .iter()
            .map(|stmt| self.stmt(stmt))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl Visitor<String, String> for AstRepr {
    fn visit_expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Binary(binary) => {
                let left = self.visit_expr(&binary.left);
                let right = self.visit_expr(&binary.right);
                let operator = binary.operator.lexeme();
                format!("Expr::Binary({left} {operator} {right})")
            }
            Expr::Unary(unary) => {
                let operator = unary.operator.lexeme();
                let right = self.visit_expr(&unary.right);
                format!("Expr::Unary({operator} {right})")
            }
            Expr::Literal(object) => match object {
                Object::Null => "nil".to_string(),
                Object::Number(n) => (*n).to_string(),
                Object::String(s) => format!("\"{s}\""),
                Object::Bool(b) => b.to_string(),
                Object::Callable(callable) => callable.to_string(),
                Object::LoxInstance(instance) => instance.to_string(),
            },
            Expr::Grouping(group) => {
                let expr = self.visit_expr(&group.expr);
                format!("Expr::Group({expr})")
            }
            Expr::Variable(var) => {
                format!("Expr::Variable({})", var.name.lexeme())
            }
            Expr::Assign(assign) => {
                let value = self.visit_expr(&assign.value);
                format!("Expr::Assign({} = {})", assign.name.lexeme(), value)
            }
            Expr::Logical(logical) => {
                let left = self.visit_expr(&logical.left);
                let right = self.visit_expr(&logical.right);
                format!(
                    "Expr::Logical({} {} {})",
                    left,
                    logical.operator.lexeme(),
                    right
                )
            }
            Expr::Call(call) => {
                let callee = self.visit_expr(&call.callee);
                let arguments = call
                    .arguments
                    .iter()
                    .map(|arg| self.visit_expr(arg))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("Expr::Call(callee={callee} arguments={arguments})")
            }
            Expr::Get(get) => {
                let object = self.visit_expr(&get.object);
                let name = get.name.lexeme();
                format!("Expr::Get(object={object}, name={name})")
            }
            Expr::Set(set) => {
                let object = self.visit_expr(&set.object);
                let name = set.name.lexeme();
                let value = self.visit_expr(&set.value);
                format!("Expr::Set(object={object}, name={name}, value={value})")
            }
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) -> String {
        match s {
            Stmt::Expression(e) => {
                let expr = self.visit_expr(e);
                format!("Stmt::Expr({expr})")
            }
            Stmt::Print(e) => {
                let value = self.visit_expr(e);
                format!("Stmt::Print({value})")
            }
            Stmt::Var(var) => {
                let name = var.identifier.lexeme();
                let value = self.visit_expr(&var.expression);
                format!("Stmt::Var({name} = {value})")
            }
            Stmt::Block(block) => {
                let mut result = String::new();
                result.push_str("Stmt::Block(");
                result.push_str(
                    &block
                        .statements
                        .iter()
                        .map(|s| self.visit_stmt(s))
                        .collect::<Vec<_>>()
                        .join(" "),
                );
                result.push(')');
                result
            }
            Stmt::If(if_statement) => {
                let condition = self.visit_expr(&if_statement.condition);
                let then_branch = self.visit_stmt(&if_statement.then_branch);
                let else_branch = if_statement
                    .else_branch
                    .as_ref()
                    .map(|s| self.visit_stmt(s));
                match else_branch {
                    Some(else_branch) => {
                        format!("Stmt::If(cond={condition} then={then_branch} else={else_branch})")
                    }
                    None => format!("Stmt::If(cond={condition} then={then_branch})"),
                }
            }
            Stmt::While(while_statement) => {
                let condition = self.visit_expr(&while_statement.condition);
                let body = self.visit_stmt(&while_statement.body);
                format!("Stmt::While(cond={condition}, body={body})")
            }
            Stmt::Function(fun) => {
                let name = fun.name.lexeme();
                let params = fun
                    .params
                    .iter()
                    .map(|token| token.lexeme())
                    .collect::<Vec<_>>()
                    .join(",");
                let body = self.visit_stmt(&fun.body);
                format!("Stmt::Function(name={name} params={params} body={body})")
            }
            Stmt::Return(return_statement) => {
                let value = self.visit_expr(&return_statement.value);
                format!("Stmt::Return({value})")
            }
            Stmt::Class(class) => {
                let methods = class
                    .methods
                    .iter()
                    .map(|fun| self.visit_stmt(fun))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "
                    Stmt::Class(name={}, methods=({methods}))",
                    class.name.lexeme()
                )
            }
        }
    }
}
