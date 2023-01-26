use std::{collections::HashMap, io::StdoutLock};

use crate::{
    callable::{Callable, LoxCallable},
    class::{LoxInstance, LoxInstanceFields},
    environment::EnvironmentTree,
    error::{reporter::ErrorReporter, runtime_error::RuntimeError},
    expr::Expr,
    function::LoxFunction,
    object::Object,
    stmt::Stmt,
    token::{Token, TokenType},
    visitor::Visitor,
};

pub(crate) struct Interpreter<W>
where
    W: std::io::Write,
{
    writer: W,
    environment: EnvironmentTree,
    errors: Vec<RuntimeError>,
    locals: HashMap<Expr, usize>,
    lox_instances: HashMap<LoxInstance, LoxInstanceFields>,
}

type InterpreterResult<T> = Result<T, RuntimeError>;

impl<W> ErrorReporter<RuntimeError> for Interpreter<W>
where
    W: std::io::Write,
{
    fn errors(&self) -> &[RuntimeError] {
        &self.errors
    }
}

#[allow(dead_code)]
impl<W> Interpreter<W>
where
    W: std::io::Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            environment: EnvironmentTree::default(),
            errors: Default::default(),
            locals: Default::default(),
            lox_instances: Default::default(),
        }
    }

    fn expr(&mut self, e: &Expr) -> InterpreterResult<Object> {
        e.walk_epxr(self)
    }

    pub fn stmt(&mut self, s: &Stmt) -> InterpreterResult<()> {
        s.walk_stmt(self)
    }

    pub fn locals(&self) -> &HashMap<Expr, usize> {
        &self.locals
    }

    pub fn resolve(&mut self, expr: Expr, depth: usize) {
        self.locals.insert(expr, depth);
    }

    pub fn lookup_variable(&self, expr: &Expr, token: &Token) -> InterpreterResult<Object> {
        let result = match self.locals.get(expr) {
            Some(depth) => self.environment.get_at(token.lexeme(), *depth),
            None => self.environment.get_global(token.lexeme()),
        };
        result.ok_or_else(|| RuntimeError::undefined_variable(token))
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

    pub fn add_new_instance(&mut self, lox_instance: LoxInstance) {
        self.lox_instances
            .insert(lox_instance.clone(), LoxInstanceFields::from(lox_instance));
    }

    pub fn instance_id(&self) -> usize {
        self.lox_instances.len()
    }

    pub fn environment_mut(&mut self) -> &mut EnvironmentTree {
        &mut self.environment
    }

    pub fn write(&mut self, s: &str) -> Result<(), std::io::Error> {
        writeln!(self.writer, "{s}")
    }
}

impl<'a> Default for Interpreter<StdoutLock<'a>> {
    fn default() -> Self {
        Self {
            writer: std::io::stdout().lock(),
            environment: EnvironmentTree::default(),
            errors: Default::default(),
            locals: Default::default(),
            lox_instances: Default::default(),
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
                let operator = &binary.operator;
                match operator.token_type() {
                    TokenType::Minus => {
                        Ok((lhs - rhs).map_err(|err| RuntimeError::from((operator, err)))?)
                    }
                    TokenType::Star => {
                        Ok((lhs * rhs).map_err(|err| RuntimeError::from((operator, err)))?)
                    }
                    TokenType::Slash => {
                        Ok((lhs / rhs).map_err(|err| RuntimeError::from((operator, err)))?)
                    }
                    TokenType::Plus => {
                        Ok((lhs + rhs).map_err(|err| RuntimeError::from((operator, err)))?)
                    }
                    TokenType::Greater => {
                        Ok((lhs.gt(&rhs)).map_err(|err| RuntimeError::from((operator, err)))?)
                    }
                    TokenType::GreaterEqual => {
                        Ok((lhs.ge(&rhs)).map_err(|err| RuntimeError::from((operator, err)))?)
                    }
                    TokenType::Less => {
                        Ok((lhs.lt(&rhs)).map_err(|err| RuntimeError::from((operator, err)))?)
                    }
                    TokenType::LessEqual => {
                        Ok((lhs.le(&rhs)).map_err(|err| RuntimeError::from((operator, err)))?)
                    }
                    TokenType::BangEqual => {
                        Ok((lhs.ne(&rhs)).map_err(|err| RuntimeError::from((operator, err)))?)
                    }
                    TokenType::EqualEqual => {
                        Ok((lhs.eq(&rhs)).map_err(|err| RuntimeError::from((operator, err)))?)
                    }
                    _ => unimplemented!(),
                }
            }
            Expr::Unary(unary) => {
                let rhs = self.visit_expr(&unary.right)?;
                let operator = &unary.operator;
                match operator.token_type() {
                    TokenType::Minus => {
                        Ok((-rhs).map_err(|err| RuntimeError::from((operator, err)))?)
                    }
                    TokenType::Bang => Ok(Object::Bool(!rhs.is_truthy())),
                    _ => unimplemented!(),
                }
            }
            Expr::Literal(object) => Ok(object.clone()),
            Expr::Grouping(group) => Ok(self.visit_expr(&group.expr)?),
            Expr::Variable(var) => self.lookup_variable(e, &var.name),
            Expr::Assign(assign) => {
                let name = &assign.name;
                let value = self.visit_expr(&assign.value)?;
                let result = match self.locals.get(e) {
                    Some(depth) => self.environment.assign_at(name.lexeme(), value, *depth),
                    None => self.environment.assign_global(name.lexeme(), value),
                };
                result.ok_or_else(|| RuntimeError::undefined_variable(name))
            }
            Expr::Logical(logical) => {
                let left = self.visit_expr(&logical.left)?;
                if logical.operator.token_type() == &TokenType::Or {
                    if left.is_truthy() {
                        return Ok(left);
                    }
                } else if !left.is_truthy() {
                    return Ok(left);
                }
                self.visit_expr(&logical.right)
            }
            Expr::Call(call) => {
                let callee = self.visit_expr(&call.callee)?;
                match callee {
                    Object::Callable(mut callee) => {
                        let arguments: InterpreterResult<Vec<_>> = call
                            .arguments
                            .iter()
                            .map(|arg| self.visit_expr(arg))
                            .collect();
                        let arguments = arguments?;
                        if arguments.len() != callee.arity() {
                            return Err(RuntimeError::number_arguments_mismatch(
                                &call.paren,
                                callee.arity(),
                                arguments.len(),
                            ));
                        }
                        callee.call(self, arguments)
                    }
                    _ => Err(RuntimeError::object_not_callable(&call.paren, &callee)),
                }
            }
            Expr::Get(get) => {
                let object = self.visit_expr(&get.object)?;
                match object {
                    Object::LoxInstance(instance_no_fields) => {
                        // we always sure that instance is exist
                        let instance = self.lox_instances.get(&instance_no_fields).unwrap();
                        instance
                            .get(&get.name)
                            .ok_or_else(|| RuntimeError::undefined_property(&get.name))
                    }
                    _ => Err(RuntimeError::only_class_instance_has_field(
                        &object, &get.name,
                    )),
                }
            }

            Expr::Set(set) => {
                let object = self.visit_expr(&set.object)?;
                match object {
                    Object::LoxInstance(instance_no_fields) => {
                        let value = self.visit_expr(&set.value)?;
                        // we always sure that instance is exist
                        let instance = self.lox_instances.get_mut(&instance_no_fields).unwrap();
                        instance.set(&set.name, value.clone());
                        Ok(value)
                    }
                    _ => Err(RuntimeError::only_class_instance_has_field(
                        &object, &set.name,
                    )),
                }
            }
            Expr::This(this) => self.lookup_variable(e, &this.keyword),
            Expr::Super(_) => todo!(),
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
                let error = block
                    .statements
                    .iter()
                    .map(|s| self.visit_stmt(s))
                    .find(|r| r.is_err());
                self.environment.move_to_outer();
                if let Some(error) = error {
                    return error;
                }
            }
            Stmt::If(if_statement) => {
                let condition = self.visit_expr(&if_statement.condition)?;
                if condition.is_truthy() {
                    self.visit_stmt(&if_statement.then_branch)?;
                } else if let Some(else_branch) = &if_statement.else_branch {
                    self.visit_stmt(else_branch)?;
                }
            }
            Stmt::While(while_statement) => loop {
                let condition = self.visit_expr(&while_statement.condition)?;
                if !condition.is_truthy() {
                    break;
                }
                self.visit_stmt(&while_statement.body)?;
            },

            Stmt::Function(fun) => {
                let closure = self.environment.clone();
                self.environment.define(
                    fun.name.lexeme(),
                    Object::Callable(LoxCallable::lox_function(fun.clone(), closure)),
                );
            }

            Stmt::Return(return_statement) => {
                let value = self.visit_expr(&return_statement.value)?;
                return Err(RuntimeError::return_value(&return_statement.keyword, value));
            }

            Stmt::Class(class) => {
                let superclass = match &class.superclass {
                    Some(superclass) => Some(self.visit_expr(&Expr::Variable(superclass.clone()))?),
                    None => None,
                };

                let superclass = match &superclass {
                    Some(inner) => match &inner {
                        &Object::Callable(LoxCallable::LoxClass(lox_class)) => {
                            Some(lox_class.clone())
                        }
                        _ => {
                            let name = &class.superclass.clone().unwrap().name;
                            return Err(RuntimeError::superclass_must_be_class(name));
                        }
                    },
                    _ => None,
                };

                let mut methods = HashMap::new();
                for method in &class.methods {
                    if let Stmt::Function(method) = method {
                        let method_name = method.name.lexeme();
                        let initializer = method_name == "init";
                        let lox_function =
                            LoxFunction::new(method.clone(), self.environment.clone(), initializer);
                        methods.insert(lox_function.name().to_string(), lox_function);
                    } else {
                        todo!("class should only contain methods")
                    }
                }
                self.environment.define(
                    class.name.lexeme(),
                    Object::Callable(LoxCallable::lox_class(class.clone(), superclass, methods)),
                )
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use std::time::SystemTime;

    use crate::{
        error::reporter::Reporter, parser::Parser, resolver::Resolver, scanner::Scanner,
        source::SourcePos,
    };

    use super::*;

    fn test_interpreter(source: &str, expected_output: &str) -> Result<(), std::io::Error> {
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

        interpreter.interpret(&statements);
        let error_msg = interpreter.error_msg(&reporter);
        interpreter.write(&error_msg)?;

        let result = String::from_utf8(result).unwrap();
        assert_eq!(result.trim(), expected_output.trim());

        Ok(())
    }

    #[test]
    fn only_number_could_be_negation() -> Result<(), std::io::Error> {
        let source = r#"
-1; 
-nil; 
-true; 
-false; 
-"a";
"#;
        let expected_output = r#"
[line 3]: RuntimeError: Could not negative non-number
-nil;
^
[line 4]: RuntimeError: Could not negative non-number
-true;
^
[line 5]: RuntimeError: Could not negative non-number
-false;
^
[line 6]: RuntimeError: Could not negative non-number
-"a";
^
"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn nil_and_false_are_false() -> Result<(), std::io::Error> {
        let source = r#"
print !1; 
print !nil;
print !true;
print !false;
print !"a";
"#;
        let expected_output = r#"
false
true
false
true
false
"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn only_number_can_subtract() -> Result<(), std::io::Error> {
        let source = r#"
print 1 - 3;
"a" - true;
true - nil;
"#;
        let expected_output = r#"
-2
[line 3]: RuntimeError: Could not subtract non-number
"a" - true;
    ^
[line 4]: RuntimeError: Could not subtract non-number
true - nil;
     ^
"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn only_number_can_multiply() -> Result<(), std::io::Error> {
        let source = r#"
print 5 * 3;
"a" * true;
true * nil;
"#;
        let expected_output = r#"
15
[line 3]: RuntimeError: Could not multiply non-number
"a" * true;
    ^
[line 4]: RuntimeError: Could not multiply non-number
true * nil;
     ^
"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn only_number_can_divide() -> Result<(), std::io::Error> {
        let source = r#"
print 6 / 3;
"a" / true;
true / nil;
"#;
        let expected_output = r#"
2
[line 3]: RuntimeError: Could not divide non-number
"a" / true;
    ^
[line 4]: RuntimeError: Could not divide non-number
true / nil;
     ^
"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn divide_by_zero() -> Result<(), std::io::Error> {
        let source = r#"
1/0;
"#;
        let expected_output = r#"
[line 2]: RuntimeError: Division by zero
1/0;
 ^
    "#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn only_number_or_string_could_add_together() -> Result<(), std::io::Error> {
        let source = r#"
print 6 + 2;
print "Hello" + " World";
true + 1;
nil + false;
"#;
        let expected_output = r#"
8
Hello World
[line 4]: RuntimeError: Could not add non-number or non-string together
true + 1;
     ^
[line 5]: RuntimeError: Could not add non-number or non-string together
nil + false;
    ^
"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn only_number_can_be_compare_using_ge_gt_le_lt() -> Result<(), std::io::Error> {
        let source = r#"
print 1 > 2;
print 1 >= 2;
print 2 < 3;
print 2 <= 2;
true > false;
"a" > "b";
"a" > false;
nil > nil;
"#;
        let expected_output = r#"
false
false
true
true
[line 6]: RuntimeError: Could not compare non-number together
true > false;
     ^
[line 7]: RuntimeError: Could not compare non-number together
"a" > "b";
    ^
[line 8]: RuntimeError: Could not compare non-number together
"a" > false;
    ^
[line 9]: RuntimeError: Could not compare non-number together
nil > nil;
    ^
"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn same_kind_object_can_be_true_or_false() -> Result<(), std::io::Error> {
        let source = r#"
print 1 == 1;
print 1 != 2;
print "Hello" == "Hello";
print "Hello" != "World";
print nil == nil;
print true == false;
"#;
        let expected_output = r#"
true
true
true
true
true
false
"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn compare_different_kind_object_always_false() -> Result<(), std::io::Error> {
        let source = r#"
print nil != true;
print 1 == true;
"#;
        let expected_output = r#"
true
false
"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn assignment() -> Result<(), std::io::Error> {
        let source = r#"
var x = 1;
x = 2;
print x;
x = y;
"#;
        let expected_output = r#"
2
[line 5]: RuntimeError: Undefined variable `y`
x = y;
    ^
"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn nested_block() -> Result<(), std::io::Error> {
        let source = r#"
var a = "global a";
var b = "global b";
var c = "global c";
{
    var a = "outer a";
    var b = "outer b";
    {
        var a = "inner a";
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
"#;

        let expected_output = r#"
inner a
outer b
global c
outer a
outer b
global c
global a
global b
global c
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn if_then_statement() -> Result<(), std::io::Error> {
        let source = r#"
if (true) 
    print "if then";
"#;
        let expected_output = r#"if then"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn if_then_else_statement() -> Result<(), std::io::Error> {
        let source = r#"
if (false) 
    print "if then";
else
    print "if then else";
"#;
        let expected_output = r#"if then else"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn nested_if_then_else() -> Result<(), std::io::Error> {
        let source = r#"
if (false)
    print "if then";
    if (true)
        print "nested if then";
    else
        print "nested if then else";
"#;
        let expected_output = r#"nested if then"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn logical() -> Result<(), std::io::Error> {
        let source = r#"
print true or 1; // true
print false or 1; // 1
print true and 1;  // 1
print false and 1; // false
print 1 and 2 and 3 or 4; // 3
"#;

        let expected_output = r#"
true
1
1
false
3
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn while_statement() -> Result<(), std::io::Error> {
        let source = r#"
var x = 1;
var y = 100;
while (x <= 5) {
    print y;
    y = y + 1;
    x = x + 1;
}
"#;

        let expected_output = r#"
100
101
102
103
104
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn function_call() -> Result<(), std::io::Error> {
        let source = r#"
fun f(x) {
    var y = 1;
    print x + y;
}

f(2);
f(5);
"#;

        let expected_output = r#"
3
6
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn function_call_arguments_mismatch() -> Result<(), std::io::Error> {
        let source = r#"
fun f(x) {print x + 1;}
f(3, 4);
"#;

        let expected_output = r#"
[line 3]: RuntimeError: Expected 1 arguments. Found 2 arguments
f(3, 4);
      ^
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn navtive_clock_function() -> Result<(), std::io::Error> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let source = format!(
            "
var x = clock();
print x >= {};
",
            now.as_millis()
        );

        let expected_output = r#"true"#;

        test_interpreter(&source, expected_output)
    }

    #[test]
    fn return_statement() -> Result<(), std::io::Error> {
        let source = r#"
// normal return
fun f1(x) {
    return x + 5;
}
print f1(2); // 7

// nested return
fun f2(x) {
    if (x > 5) 
        return 5;
    else 
        return x;
}
print f2(8); // 5
print f2(1); // 1

// no return
fun f2(x) {
    print 3;
}
print f2(5); // 3 and nothing
"#;

        let expected_output = r#"
7
5
1
3
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn fibonacci() -> Result<(), std::io::Error> {
        let source = r#"
fun fib(n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

for (var i = 1; i < 10; i = i + 1) {
    print fib(i);
}
"#;

        let expected_output = r#"
1
1
2
3
5
8
13
21
34
"#;
        test_interpreter(source, expected_output)
    }

    #[test]
    fn closure() -> Result<(), std::io::Error> {
        let source = r#"
fun makeCounter() {
  var i = 0;
  fun count() {
    i = i + 1;
    print i;
  }
  return count;
}
var counter = makeCounter();
counter(); // 1.
counter(); // 2.
"#;

        let expected_output = r#"
1
2
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn class_declaration() -> Result<(), std::io::Error> {
        let source = r#"
class DevonshireCream {
  serveOn() {
    return "Scones";
  }
}
print DevonshireCream;
"#;

        let expected_output = r#"
<class DevonshireCream>
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn class_instance_declaration() -> Result<(), std::io::Error> {
        let source = r#"
class Hello {}
var instance1 = Hello();
print instance1;
var instance2 = Hello();
print instance2;
var instance3 = Hello();
print instance3;
"#;

        let expected_output = r#"
<Hello instance, id 0>
<Hello instance, id 1>
<Hello instance, id 2>
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn get_field_on_non_class_instance() -> Result<(), std::io::Error> {
        let source = r#"
var x = 1;
x.name;
"#;

        let expected_output = r#"
[line 3]: RuntimeError: `1` is not class instance. It cannot have field `name`
x.name;
  ^^^^
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn set_field_on_non_class_instance() -> Result<(), std::io::Error> {
        let source = r#"
var x = 1;
x.name = 2;
"#;

        let expected_output = r#"
[line 3]: RuntimeError: `1` is not class instance. It cannot have field `name`
x.name = 2;
  ^^^^
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn undefined_property() -> Result<(), std::io::Error> {
        let source = r#"
class Hello {}
var hello = Hello();
print hello.name;
"#;

        let expected_output = r#"
[line 4]: RuntimeError: Undefined property `name`
print hello.name;
            ^^^^
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn get_set_field_attribute_on_class_instance() -> Result<(), std::io::Error> {
        let source = r#"
class Hello {}
var hello = Hello();
hello.name = "dqk";
print hello.name;
"#;

        let expected_output = r#"
dqk
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn method_on_class() -> Result<(), std::io::Error> {
        let source = r#"
class Bacon {
    eat() {
      print "Crunch crunch crunch!";
    }
}
  
Bacon().eat(); // Prints "Crunch crunch crunch!".
"#;

        let expected_output = r#"
Crunch crunch crunch!
        "#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn this_bind_to_variable() -> Result<(), std::io::Error> {
        let source = r#"
class Cake {
  taste() {
    var adjective = "delicious";
    print "The " + this.flavor + " cake is " + adjective + "!";
  }
}

var cake = Cake();
cake.flavor = "German chocolate";
cake.taste(); // Prints "The German chocolate cake is delicious!".
"#;

        let expected_output = r#"
The German chocolate cake is delicious!
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn this_support_closure() -> Result<(), std::io::Error> {
        let source = r#"
class Human {
  getCallback() {
    fun localFunction() {
      print this.name;
    }

    return localFunction;
  }
}

var thing = Human();
thing.name = "dqk";
var callback = thing.getCallback();
callback();
"#;

        let expected_output = r#"
dqk
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn call_init_inside_class() -> Result<(), std::io::Error> {
        let source = r#"
class Hello {
    init(name) {
        print "hello " + name;
    }
}
var hello = Hello("dqk");
"#;

        let expected_output = r#"
hello dqk
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn implicit_return_this() -> Result<(), std::io::Error> {
        let source = r#"
class Foo {
  init() {
    print this;
  }
}

var foo = Foo();
print foo.init();
"#;

        let expected_output = r#"
<Foo instance, id 0>
<Foo instance, id 0>
<Foo instance, id 0>
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn empty_return_inside_init_should_return_this() -> Result<(), std::io::Error> {
        let source = r#"
class Foo {
  init() {
    this.x = 1;
    return;
  }
}

var foo = Foo();
print foo.x;
"#;

        let expected_output = r#"
1
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn empty_return_inside_init_should_return_early() -> Result<(), std::io::Error> {
        let source = r#"
class Foo {
  init() {
    return;
    this.x = 1;
  }
}

var foo = Foo();
print foo.x;
"#;

        let expected_output = r#"
[line 10]: RuntimeError: Undefined property `x`
print foo.x;
          ^
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn superclass_must_be_class() -> Result<(), std::io::Error> {
        let source = r#"
class Hello : NotAClass {}
"#;

        let expected_output = r#"
[line 2]: RuntimeError: Undefined variable `NotAClass`
class Hello : NotAClass {}
              ^^^^^^^^^
"#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn subclass_can_inherit_valid_superclass() -> Result<(), std::io::Error> {
        let source = r#"
class A {}
class B : A {}
class C : B {}
class D : A {}
"#;

        let expected_output = r#""#;

        test_interpreter(source, expected_output)
    }

    #[test]
    fn subclass_inherit_method_from_superclass() -> Result<(), std::io::Error> {
        let source = r#"
class A {
    f() {
        print "`f` is called from super class";
    }

    g() {
        print "`g` is called from super class";
    }
}

class B : A {
    g() {
        print "`g` is called from sub class";
    }
}

B().g(); // call g from sub
B().f(); // call f from super
"#;

        let expected_output = r#"
`g` is called from sub class
`f` is called from super class
"#;

        test_interpreter(source, expected_output)
    }
}
