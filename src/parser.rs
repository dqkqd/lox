use std::{iter::Peekable, vec::IntoIter};

const MAXIMUM_ARGUMENTS: usize = 255;

use crate::{
    error::{parse_error::ParseError, reporter::ErrorReporter},
    expr::{Assign, Binary, Call, Expr, Grouping, Unary, Variable},
    object::Object,
    scanner::Scanner,
    stmt::{Block, Function, If, Return, Stmt, Var, While},
    token::{Token, TokenType},
};

type ParseResult<T> = Result<T, ParseError>;

pub(crate) struct Parser {
    it: Peekable<IntoIter<Token>>,
    errors: Vec<ParseError>,
}

impl From<&Scanner> for Parser {
    fn from(scanner: &Scanner) -> Self {
        Parser::new(scanner.tokens())
    }
}

impl ErrorReporter<ParseError> for Parser {
    fn errors(&self) -> &[ParseError] {
        &self.errors
    }
}

impl Parser {
    #[allow(clippy::unnecessary_to_owned)]
    fn new(tokens: &[Token]) -> Self {
        Parser {
            it: tokens.to_vec().into_iter().peekable(),
            errors: Vec::new(),
        }
    }

    fn is_end(&self) -> bool {
        self.it.len() == 1 // the last one if eof
    }

    fn peek(&mut self) -> &Token {
        // we always have 1 element left, so we can safety unwrap
        self.it.peek().unwrap()
    }

    fn peek_type(&mut self) -> &TokenType {
        self.peek().token_type()
    }

    fn next(&mut self) -> Option<Token> {
        if self.is_end() {
            None
        } else {
            self.it.next()
        }
    }

    fn match_peek_type(&mut self, tokens_type: &[TokenType]) -> bool {
        let token_type = self.peek_type();
        tokens_type.iter().any(|lexeme| lexeme == token_type)
    }

    fn match_peek_type_then_advance(&mut self, tokens_type: &[TokenType]) -> Option<Token> {
        if self.match_peek_type(tokens_type) {
            self.next()
        } else {
            None
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        loop {
            if self.is_end() {
                break;
            }

            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    if err.panic() {
                        self.synchronize();
                    }
                    self.errors.push(err)
                }
            }
        }
        statements
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        if self
            .match_peek_type_then_advance(&[TokenType::Fun])
            .is_some()
        {
            self.fun_declaration()
        } else if self
            .match_peek_type_then_advance(&[TokenType::Var])
            .is_some()
        {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn fun_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume_identifier("function name")?;
        self.consume(TokenType::LeftParen)?;
        let mut params = Vec::new();
        if self.peek_type() != &TokenType::RightParen {
            loop {
                let param = self.consume_identifier("parameter name")?;
                params.push(param);
                if params.len() >= MAXIMUM_ARGUMENTS {
                    return Err(ParseError::maximum_arguments(
                        self.peek(),
                        MAXIMUM_ARGUMENTS,
                    ));
                }
                if self.consume(TokenType::Comma).is_err() {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen)?;
        self.consume(TokenType::LeftBrace)?;
        let body = self.block()?;
        Ok(Stmt::Function(Function::new(name, params, body)))
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        match self.peek_type() {
            TokenType::Identifier(_) => {
                let token = self.next().unwrap();
                let initializer = match self
                    .match_peek_type_then_advance(&[TokenType::Equal])
                    .is_some()
                {
                    true => self.expression()?,
                    false => Expr::Literal(Object::Null),
                };
                self.consume(TokenType::Semicolon)?;
                Ok(Stmt::Var(Var::new(token, initializer)))
            }
            _ => {
                let error = ParseError::unexpected_token(
                    self.peek(),
                    &TokenType::Identifier("variable name".to_string()),
                );
                Err(error)
            }
        }
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        match self.peek_type() {
            TokenType::If => {
                self.next();
                self.if_statement()
            }
            TokenType::Print => {
                self.next();
                self.print_statement()
            }
            TokenType::Return => {
                // we need keyword return to find the line
                // so we don't call self.next() here
                self.return_statement()
            }
            TokenType::While => {
                self.next();
                self.while_statement()
            }
            TokenType::For => {
                self.next();
                self.for_statement()
            }
            TokenType::LeftBrace => {
                self.next();
                self.block()
            }
            _ => self.expression_statement(),
        }
    }

    fn return_statement(&mut self) -> ParseResult<Stmt> {
        let keyword = self.consume(TokenType::Return)?;
        let value = match self.peek_type() {
            TokenType::Semicolon => Expr::Literal(Object::Null),
            _ => self.expression()?,
        };
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Return(Return::new(keyword, value)))
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Print(expr))
    }

    fn block(&mut self) -> ParseResult<Stmt> {
        let mut statements = Vec::new();
        loop {
            if self.is_end() || self.peek_type() == &TokenType::RightBrace {
                break;
            }
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace)?;
        Ok(Stmt::Block(Block::new(statements)))
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Expression(expr))
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen)?;

        let then_branch = self.declaration()?;

        let else_branch = match self.peek_type() {
            TokenType::Else => {
                self.next();
                Some(self.declaration()?)
            }
            _ => None,
        };

        Ok(Stmt::If(If::new(condition, then_branch, else_branch)))
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen)?;
        let body = self.declaration()?;
        Ok(Stmt::While(While::new(condition, body)))
    }

    fn for_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen)?;
        let initializer = match self.peek_type() {
            TokenType::Semicolon => {
                self.next();
                None
            }
            TokenType::Var => {
                self.next();
                Some(self.var_declaration())
            }
            _ => Some(self.expression_statement()),
        };

        let condition = match self.peek_type() {
            TokenType::Semicolon => None,
            _ => Some(self.expression()?),
        };
        self.consume(TokenType::Semicolon)?;

        let increment = match self.peek_type() {
            TokenType::RightParen => None,
            _ => Some(self.expression()),
        };
        self.consume(TokenType::RightParen)?;

        let body = self.statement()?;

        // attach increment to tail of the body
        let body = match increment {
            None => body,
            Some(inc) => {
                let inc = Stmt::Expression(inc?);
                Stmt::Block(Block::new(vec![body, inc]))
            }
        };

        // make condition true when it wasn't specified
        let condition = condition.unwrap_or(Expr::Literal(Object::Bool(true)));

        // make a while loop
        let while_statement = Stmt::While(While::new(condition, body));

        // attach initializer at the head of the while statement
        let for_statement = match initializer {
            None => while_statement,
            Some(init) => Stmt::Block(Block::new(vec![init?, while_statement])),
        };

        Ok(for_statement)
    }

    // @todo this method currently pub, move this to private after all stmts are added
    pub fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.logic_or()?;

        if let Some(equal) = self.match_peek_type_then_advance(&[TokenType::Equal]) {
            let value = self.assignment()?;
            if let Expr::Variable(var) = expr {
                Ok(Expr::Assign(Assign::new(var.name, value)))
            } else {
                Err(ParseError::invalid_assignment(&equal).without_panic())
            }
        } else {
            Ok(expr)
        }
    }

    fn logic_or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.logical_and()?;
        while let Some(operator) = self.match_peek_type_then_advance(&[TokenType::Or]) {
            let rhs = self.logical_and()?;
            expr = Expr::Logical(Binary::new(expr, operator, rhs));
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;
        while let Some(operator) = self.match_peek_type_then_advance(&[TokenType::And]) {
            let rhs = self.equality()?;
            expr = Expr::Logical(Binary::new(expr, operator, rhs));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparision()?;
        while let Some(operator) =
            self.match_peek_type_then_advance(&[TokenType::BangEqual, TokenType::EqualEqual])
        {
            let right = self.comparision()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term()?;
        while let Some(operator) = self.match_peek_type_then_advance(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let right = self.term()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor()?;
        while let Some(operator) =
            self.match_peek_type_then_advance(&[TokenType::Minus, TokenType::Plus])
        {
            let right = self.factor()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;
        while let Some(operator) =
            self.match_peek_type_then_advance(&[TokenType::Slash, TokenType::Star])
        {
            let right = self.unary()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        if let Some(operator) =
            self.match_peek_type_then_advance(&[TokenType::Bang, TokenType::Minus])
        {
            let right = self.unary()?;
            Ok(Expr::Unary(Unary::new(operator, right)))
        } else {
            Ok(self.call()?)
        }
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.consume(TokenType::LeftParen).is_ok() {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        let mut arguments = Vec::new();
        if self.peek_type() != &TokenType::RightParen {
            loop {
                let arg = self.expression()?;
                arguments.push(arg);
                if arguments.len() >= MAXIMUM_ARGUMENTS {
                    return Err(ParseError::maximum_arguments(
                        self.peek(),
                        MAXIMUM_ARGUMENTS,
                    ));
                }
                if self.consume(TokenType::Comma).is_err() {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RightParen)?;
        Ok(Expr::Call(Call::new(callee, paren, arguments)))
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        let expr = match self.peek_type() {
            TokenType::Nil => Expr::Literal(Object::Null),
            TokenType::False => Expr::Literal(Object::Bool(false)),
            TokenType::True => Expr::Literal(Object::Bool(true)),
            TokenType::Number(number) => Expr::Literal(Object::Number(*number)),
            TokenType::String(string) => Expr::Literal(Object::String(string.clone())),
            TokenType::LeftParen => {
                self.next();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen)?;
                return Ok(Expr::Grouping(Grouping::new(expr)));
            }
            TokenType::Identifier(_) => Expr::Variable(Variable::new(self.peek().clone())),
            _ => {
                let error = ParseError::expected_expression(self.peek());
                return Err(error);
            }
        };
        self.next();
        Ok(expr)
    }

    fn consume_identifier(&mut self, ident_info: &str) -> ParseResult<Token> {
        if let TokenType::Identifier(_) = self.peek_type() {
            let ident = self.next().unwrap();
            Ok(ident)
        } else {
            Err(ParseError::unexpected_token(
                self.peek(),
                &TokenType::Identifier(ident_info.to_string()),
            ))
        }
    }

    fn consume(&mut self, token_type: TokenType) -> ParseResult<Token> {
        if self.peek_type() == &token_type {
            Ok(self.next().unwrap())
        } else {
            Err(ParseError::unexpected_token(self.peek(), &token_type))
        }
    }

    fn synchronize(&mut self) {
        // we explicited call next because we push token back after error
        let start_token_type = [
            TokenType::Class,
            TokenType::Fun,
            TokenType::Var,
            TokenType::For,
            TokenType::If,
            TokenType::While,
            TokenType::Print,
            TokenType::Return,
        ];

        loop {
            if self.is_end() || self.match_peek_type(&start_token_type) {
                break;
            }
            if &TokenType::Semicolon == self.peek_type() {
                self.next(); // eat semicolon
                break;
            }
            self.next();
        }
    }
}

#[cfg(test)]
mod test {

    use std::io::Write;

    use crate::{ast_repr::AstRepr, error::reporter::Reporter, source::SourcePos};

    use super::*;

    fn test_parser(source: &str, expected_output: &str) -> Result<(), std::io::Error> {
        let source_pos = SourcePos::new(source);
        let reporter = Reporter::new(&source_pos);
        let mut result = Vec::new();

        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();
        writeln!(&mut result, "{}", scanner.error_msg(&reporter))?;

        let mut parser = Parser::from(&scanner);
        let mut ast_repr = AstRepr::default();
        let statements = parser.parse();
        writeln!(&mut result, "{}", ast_repr.repr(&statements))?;

        writeln!(&mut result, "{}", parser.error_msg(&reporter))?;

        let result = String::from_utf8(result).unwrap();
        assert_eq!(result.trim(), expected_output.trim());

        Ok(())
    }

    #[test]
    fn primary() -> Result<(), std::io::Error> {
        let source = r#"
nil; true; false; "this is string";
123; 123.456; (nil); variable;
"#;
        let expected_output = r#"
Stmt::Expr(nil)
Stmt::Expr(true)
Stmt::Expr(false)
Stmt::Expr("this is string")
Stmt::Expr(123)
Stmt::Expr(123.456)
Stmt::Expr(Expr::Group(nil))
Stmt::Expr(Expr::Variable(variable))
"#;
        test_parser(source, expected_output)
    }

    #[test]
    fn grouping_must_be_closed() -> Result<(), std::io::Error> {
        let source = r#"(1"#;
        let expected_output = r#"
[line 1]: ParseError: Expected `)`. Found `EOF`
(1
 ^
"#;
        test_parser(source, expected_output)
    }

    #[test]
    fn unary() -> Result<(), std::io::Error> {
        let source = r#"
-1.2; !1.2; 
-"a"; !"a";
-nil; !nil;
-true; !true;
-false; !false;
-(1.2); !(1.2);
-x; !x;
"#;

        let expected_output = r#" 
Stmt::Expr(Expr::Unary(- 1.2))
Stmt::Expr(Expr::Unary(! 1.2))
Stmt::Expr(Expr::Unary(- "a"))
Stmt::Expr(Expr::Unary(! "a"))
Stmt::Expr(Expr::Unary(- nil))
Stmt::Expr(Expr::Unary(! nil))
Stmt::Expr(Expr::Unary(- true))
Stmt::Expr(Expr::Unary(! true))
Stmt::Expr(Expr::Unary(- false))
Stmt::Expr(Expr::Unary(! false))
Stmt::Expr(Expr::Unary(- Expr::Group(1.2)))
Stmt::Expr(Expr::Unary(! Expr::Group(1.2)))
Stmt::Expr(Expr::Unary(- Expr::Variable(x)))
Stmt::Expr(Expr::Unary(! Expr::Variable(x)))
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn binary() -> Result<(), std::io::Error> {
        let source = r#"
1+2; 3-7; true*false; nil/nil;
"a" == "b"; nil != nil; 3 > 7; true >= false; 2 < 3; true <= false; 
"#;

        let expected_output = r#"
Stmt::Expr(Expr::Binary(1 + 2))
Stmt::Expr(Expr::Binary(3 - 7))
Stmt::Expr(Expr::Binary(true * false))
Stmt::Expr(Expr::Binary(nil / nil))
Stmt::Expr(Expr::Binary("a" == "b"))
Stmt::Expr(Expr::Binary(nil != nil))
Stmt::Expr(Expr::Binary(3 > 7))
Stmt::Expr(Expr::Binary(true >= false))
Stmt::Expr(Expr::Binary(2 < 3))
Stmt::Expr(Expr::Binary(true <= false))
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn assignment() -> Result<(), std::io::Error> {
        let source = r#"
x = 1; x = "string"; x = true; x = nil; x = y; 
x = y
"#;
        let expected_output = r#"
Stmt::Expr(Expr::Assign(x = 1))
Stmt::Expr(Expr::Assign(x = "string"))
Stmt::Expr(Expr::Assign(x = true))
Stmt::Expr(Expr::Assign(x = nil))
Stmt::Expr(Expr::Assign(x = Expr::Variable(y)))
[line 3]: ParseError: Expected `;`. Found `EOF`
x = y
     ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn synchronize_with_semicolon() -> Result<(), std::io::Error> {
        // synchronize until semicolon, the next token should be `true`.
        let source = r#"
(1 + 2 nothing; 
true < false;
"#;
        let expected_output = r#"
Stmt::Expr(Expr::Binary(true < false))
[line 2]: ParseError: Expected `)`. Found `nothing`
(1 + 2 nothing;
       ^^^^^^^
"#;
        test_parser(source, expected_output)
    }

    #[test]
    fn synchronize_without_semicolon() -> Result<(), std::io::Error> {
        // synchronize until semicolon, `1` should be eaten, the next token should be `var`.
        let source = r#"(1 + 2 1 var"#;

        let expected_output = r#"
[line 1]: ParseError: Expected `)`. Found `1`
(1 + 2 1 var
       ^
[line 1]: ParseError: Expected `variable name`. Found `EOF`
(1 + 2 1 var
           ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn multiexpressions_with_errors() -> Result<(), std::io::Error> {
        let source = r#"
"has semicolon";
("no right paren";
("has right paren");
"no semicolon"
"#;

        let expected_output = r#"
Stmt::Expr("has semicolon")
Stmt::Expr(Expr::Group("has right paren"))
[line 3]: ParseError: Expected `)`. Found `;`
("no right paren";
                 ^
[line 5]: ParseError: Expected `;`. Found `EOF`
"no semicolon"
              ^
"#;
        test_parser(source, expected_output)
    }

    #[test]
    fn print_expression_without_semicolon() -> Result<(), std::io::Error> {
        let source = r#"
print "statement";
print "statement without semicolon"
print 1 + 2;
"#;

        let expected_output = r#"
Stmt::Print("statement")
Stmt::Print(Expr::Binary(1 + 2))
[line 4]: ParseError: Expected `;`. Found `print`
print 1 + 2;
^^^^^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn variable_declaration_statement() -> Result<(), std::io::Error> {
        let source = r#"
var x = 1; 
var x = y + 1;
var x
print x;
"#;

        let expected_output = r#"
Stmt::Var(x = 1)
Stmt::Var(x = Expr::Binary(Expr::Variable(y) + 1))
Stmt::Print(Expr::Variable(x))
[line 5]: ParseError: Expected `;`. Found `print`
print x;
^^^^^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn assignment_statement() -> Result<(), std::io::Error> {
        let source = r#"
var x = 1;
x = 2;
x = y;
"this is not assignment" = 2
"#;

        let expected_output = r#"
Stmt::Var(x = 1)
Stmt::Expr(Expr::Assign(x = 2))
Stmt::Expr(Expr::Assign(x = Expr::Variable(y)))
[line 5]: ParseError: Inavalid assignment target.
"this is not assignment" = 2
                         ^
"#;
        test_parser(source, expected_output)
    }

    #[test]
    fn assignment_statement_dont_run_to_panic_mode() -> Result<(), std::io::Error> {
        let source = r#"
2 = 1 // this has error
"this token should not be eaten";
true;
"#;

        let expected_output = r#"
Stmt::Expr("this token should not be eaten")
Stmt::Expr(true)
[line 2]: ParseError: Inavalid assignment target.
2 = 1 // this has error
  ^
"#;
        test_parser(source, expected_output)
    }

    #[test]
    fn block_statement() -> Result<(), std::io::Error> {
        let source = r#"
// nested block
{
  {
    var x = 1;
  }

  var x = 2;
}

{
  1 + 2;
"#;

        let expected_output = r#"
Stmt::Block(Stmt::Block(Stmt::Var(x = 1)) Stmt::Var(x = 2))
[line 12]: ParseError: Expected `}`. Found `EOF`
  1 + 2;
        ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn if_statement() -> Result<(), std::io::Error> {
        let source = r#"
// normal
if (true) var x = 1;
else var x = 2;

// nested
if (1) 
  if (2) 3;
  else 4;
"#;

        let expected_output = r#"
Stmt::If(cond=true then=Stmt::Var(x = 1) else=Stmt::Var(x = 2))
Stmt::If(cond=1 then=Stmt::If(cond=2 then=Stmt::Expr(3) else=Stmt::Expr(4)))
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn logical_or() -> Result<(), std::io::Error> {
        let source = r#"
1 or 2;
1 or 2 or 3;
1 and 2 or 3;
"#;

        let expected_output = r#"
Stmt::Expr(Expr::Logical(1 or 2))
Stmt::Expr(Expr::Logical(Expr::Logical(1 or 2) or 3))
Stmt::Expr(Expr::Logical(Expr::Logical(1 and 2) or 3))
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn while_statement() -> Result<(), std::io::Error> {
        let source = r#"
while (1 + 2)
print 1;

while (1 + 2
"#;

        let expected_output = r#"
Stmt::While(cond=Expr::Binary(1 + 2), body=Stmt::Print(1))
[line 5]: ParseError: Expected `)`. Found `EOF`
while (1 + 2
            ^
"#;
        test_parser(source, expected_output)
    }

    #[test]
    fn normal_for_statement() -> Result<(), std::io::Error> {
        let source = r#"
for (var i = 0; i < 5; i = i + 1)
  print i;
for (1;2;3)
  4;
"#;

        let expected_output = r#"
Stmt::Block(Stmt::Var(i = 0) Stmt::While(cond=Expr::Binary(Expr::Variable(i) < 5), body=Stmt::Block(Stmt::Print(Expr::Variable(i)) Stmt::Expr(Expr::Assign(i = Expr::Binary(Expr::Variable(i) + 1))))))
Stmt::Block(Stmt::Expr(1) Stmt::While(cond=2, body=Stmt::Block(Stmt::Expr(4) Stmt::Expr(3))))
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn missing_parts_for_statement() -> Result<(), std::io::Error> {
        let source = r#"
for (; 2; 3) 4; // missing initializer
for (1; ; 3) 4; // missing condition
for (1; 2; ) 4; // missing increment
for (;;) 4;    // miss all      
"#;

        let expected_output = r#"
Stmt::While(cond=2, body=Stmt::Block(Stmt::Expr(4) Stmt::Expr(3)))
Stmt::Block(Stmt::Expr(1) Stmt::While(cond=true, body=Stmt::Block(Stmt::Expr(4) Stmt::Expr(3))))
Stmt::Block(Stmt::Expr(1) Stmt::While(cond=2, body=Stmt::Expr(4)))
Stmt::While(cond=true, body=Stmt::Expr(4))
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn for_statement_without_semicolon() -> Result<(), std::io::Error> {
        let source = r#"
for (;) 4; // missing semicolon
for () 2; // no semicolon
"#;

        let expected_output = r#"
[line 2]: ParseError: Expected expression
for (;) 4; // missing semicolon
      ^
[line 3]: ParseError: Expected expression
for () 2; // no semicolon
     ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn for_statement_without_right_paren() -> Result<(), std::io::Error> {
        let source = r#"
// no right paren
for (
"#;

        let expected_output = r#"
[line 3]: ParseError: Expected expression
for (
     ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn for_statement_without_right_paren_but_already_parsed_init_cond_inc(
    ) -> Result<(), std::io::Error> {
        let source = r#"
// no right paren but already parsed init, cond and inc
for (;;
"#;

        let expected_output = r#"
[line 3]: ParseError: Expected `)`. Found `EOF`
for (;;
       ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn for_statement_without_left_paren() -> Result<(), std::io::Error> {
        let source = r#"
// no left paren
for )
"#;

        let expected_output = r#"
[line 3]: ParseError: Expected `(`. Found `)`
for )
    ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn normal_function_declaration() -> Result<(), std::io::Error> {
        let source = r#"
fun hello(x, y, z) {
  print x;
  print y;
  print z;
}
"#;

        let expected_output = r#"
Stmt::Function(name=hello params=x,y,z body=Stmt::Block(Stmt::Print(Expr::Variable(x)) Stmt::Print(Expr::Variable(y)) Stmt::Print(Expr::Variable(z))))
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn function_declaration_missing_functino_name() -> Result<(), std::io::Error> {
        let source = r#"
// missing function name
fun (; 
"#;

        let expected_output = r#"
[line 3]: ParseError: Expected `function name`. Found `(`
fun (;
    ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn function_declaration_missing_left_paren() -> Result<(), std::io::Error> {
        let source = r#"
// missing left paren
fun f); 
"#;

        let expected_output = r#"
[line 3]: ParseError: Expected `(`. Found `)`
fun f);
     ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn function_declaration_missing_right_paren() -> Result<(), std::io::Error> {
        let source = r#"
// missing right paren
fun f(x, y; 
"#;

        let expected_output = r#"
[line 3]: ParseError: Expected `)`. Found `;`
fun f(x, y;
          ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn function_declaration_missing_parameter_name() -> Result<(), std::io::Error> {
        let source = r#"
// missing parameter name
fun f(,); 
"#;

        let expected_output = r#"
[line 3]: ParseError: Expected `parameter name`. Found `,`
fun f(,);
      ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn function_declaration_missing_body() -> Result<(), std::io::Error> {
        let source = r#"
// missing body
fun f(); 
"#;

        let expected_output = r#"
[line 3]: ParseError: Expected `{`. Found `;`
fun f();
       ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn function_declaration_missing_left_brace() -> Result<(), std::io::Error> {
        let source = r#"
// missing left brace
fun f()}; 
"#;

        let expected_output = r#"
[line 3]: ParseError: Expected `{`. Found `}`
fun f()};
       ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn function_declaration_missing_right_brace() -> Result<(), std::io::Error> {
        let source = r#"
// missing right brace
fun f(){ print x;
"#;

        let expected_output = r#"
[line 3]: ParseError: Expected `}`. Found `EOF`
fun f(){ print x;
                 ^
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn function_with_maximum_arguments() -> Result<(), std::io::Error> {
        let mut params = Vec::new();
        while params.len() < MAXIMUM_ARGUMENTS {
            params.push("x")
        }
        let params = params.join(",");
        let source = format!("fun hello({}) {{}}", params);

        let mut indicated_error = vec![' '; source.len() - 4]; // `) {}` remove offset for braces
        indicated_error.push('^');
        let indicated_error = indicated_error.into_iter().collect::<String>();

        let expected_output = format!(
            "
[line 1]: ParseError: Could not have more than 255 arguments
{}\n{}",
            source, indicated_error
        );

        test_parser(&source, &expected_output)
    }

    #[test]
    fn normal_function_call() -> Result<(), std::io::Error> {
        let source = r#"hello("world"); // this call fuction `hello`"#;
        let expected_output = r#"
Stmt::Expr(Expr::Call(callee=Expr::Variable(hello) arguments="world"))
"#;
        test_parser(source, expected_output)
    }

    #[test]
    fn missing_parts_function_call() -> Result<(), std::io::Error> {
        let source = r#"
// missing left paren
hello);

// missing right paren
hello(1, 2; 

// missing parameter name
hello(,); 
"#;

        let expected_output = r#"
[line 3]: ParseError: Expected `;`. Found `)`
hello);
     ^
[line 6]: ParseError: Expected `)`. Found `;`
hello(1, 2;
          ^
[line 9]: ParseError: Expected expression
hello(,);
      ^ 
"#;

        test_parser(source, expected_output)
    }

    #[test]
    fn return_statement() -> Result<(), std::io::Error> {
        let source = "
fun f(x) {
return x;
}

fun f(x) {
return;
}
";

        let expected_output = "
Stmt::Function(name=f params=x body=Stmt::Block(Stmt::Return(Expr::Variable(x))))
Stmt::Function(name=f params=x body=Stmt::Block(Stmt::Return(nil)))
";

        test_parser(source, expected_output)
    }
}
