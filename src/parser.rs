use std::vec::IntoIter;

use crate::{
    error::parse_error::ParseError,
    expr::{Assign, Binary, Expr, Grouping, Unary, Variable},
    object::Object,
    scanner::Scanner,
    stmt::{Block, Stmt, Var},
    token::{Token, TokenType},
};

type ParseResult<T> = Result<T, ParseError>;

pub(crate) struct Parser {
    it: IntoIter<Token>,
    buffer: Vec<Token>,
    errors: Vec<ParseError>,
    eof_token: Token,
}

impl From<&Scanner> for Parser {
    fn from(scanner: &Scanner) -> Self {
        Parser::new(scanner.tokens())
    }
}

impl Parser {
    fn new(tokens: &[Token]) -> Self {
        let mut tokens: Vec<Token> = tokens.to_vec();
        let _eof_token = tokens
            .pop()
            .expect("TokenType::Eof must be the end of scan result");
        Parser {
            it: tokens.into_iter(),
            buffer: Vec::with_capacity(16),
            errors: Vec::new(),
            eof_token: _eof_token,
        }
    }

    pub fn had_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    fn is_end(&self) -> bool {
        self.buffer.is_empty() && self.it.len() == 0
    }

    fn prev(&mut self, token: Token) {
        self.buffer.push(token)
    }

    fn next(&mut self) -> Option<Token> {
        if !self.buffer.is_empty() {
            self.buffer.pop()
        } else {
            self.it.next()
        }
    }

    fn match_token_type(&mut self, token_type: &[TokenType]) -> Option<Token> {
        if let Some(token) = self.next() {
            let contain = token_type.iter().any(|lexeme| lexeme == token.token_type());
            if !contain {
                self.prev(token);
                None
            } else {
                Some(token)
            }
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
        if self.match_token_type(&[TokenType::Var]).is_some() {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        if let Some(token) = self.next() {
            if token.token_type().is_identifier() {
                let initializer = match self.match_token_type(&[TokenType::Equal]).is_some() {
                    true => self.expression()?,
                    false => Expr::Literal(Object::Null),
                };
                self.consume(TokenType::Semicolon)?;
                Ok(Stmt::Var(Var::new(token, initializer)))
            } else {
                let error = ParseError::unexpected_token(
                    token.line(),
                    token.token_type(),
                    &TokenType::Identifier("variable name".to_string()),
                );
                self.prev(token);
                Err(error)
            }
        } else {
            Err(ParseError::expected_expression(self.eof_token.line()))
        }
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.match_token_type(&[TokenType::Print]).is_some() {
            self.print_statement()
        } else if self.match_token_type(&[TokenType::LeftBrace]).is_some() {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Print(expr))
    }

    fn block(&mut self) -> ParseResult<Stmt> {
        let mut statements = Vec::new();
        while let Some(token) = self.next() {
            if token.token_type() == &TokenType::RightBrace {
                self.prev(token);
                break;
            }
            self.prev(token);
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

    // @todo this method currently pub, move this to private after all stmts are added
    pub fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.equality()?;

        if let Some(equal) = self.match_token_type(&[TokenType::Equal]) {
            let value = self.assignment()?;
            if let Expr::Variable(var) = expr {
                Ok(Expr::Assign(Assign::new(var.name, value)))
            } else {
                Err(ParseError::invalid_assignment(equal.line()).without_panic())
            }
        } else {
            Ok(expr)
        }
    }

    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparision()?;
        while let Some(operator) =
            self.match_token_type(&[TokenType::BangEqual, TokenType::EqualEqual])
        {
            let right = self.comparision()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term()?;
        while let Some(operator) = self.match_token_type(&[
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
        while let Some(operator) = self.match_token_type(&[TokenType::Minus, TokenType::Plus]) {
            let right = self.factor()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;
        while let Some(operator) = self.match_token_type(&[TokenType::Slash, TokenType::Star]) {
            let right = self.unary()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        if let Some(operator) = self.match_token_type(&[TokenType::Bang, TokenType::Minus]) {
            let right = self.unary()?;
            Ok(Expr::Unary(Unary::new(operator, right)))
        } else {
            Ok(self.primary()?)
        }
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        if let Some(token) = self.next() {
            match TokenType::from(token.clone()) {
                TokenType::Nil => Ok(Expr::Literal(Object::Null)),
                TokenType::False => Ok(Expr::Literal(Object::Bool(false))),
                TokenType::True => Ok(Expr::Literal(Object::Bool(true))),
                TokenType::Number(number) => Ok(Expr::Literal(Object::Number(number))),
                TokenType::String(string) => Ok(Expr::Literal(Object::String(string))),
                TokenType::LeftParen => {
                    let expr = self.expression()?;
                    self.consume(TokenType::RightParen)?;
                    Ok(Expr::Grouping(Grouping::new(expr)))
                }
                TokenType::Identifier(_) => Ok(Expr::Variable(Variable::new(token))),
                _ => {
                    let error = ParseError::expected_expression(token.line());
                    self.prev(token);
                    Err(error)
                }
            }
        } else {
            Err(ParseError::expected_expression(self.eof_token.line()))
        }
    }

    fn consume(&mut self, token_type: TokenType) -> ParseResult<()> {
        if let Some(token) = self.next() {
            if token.token_type() != &token_type {
                let error =
                    ParseError::unexpected_token(token.line(), token.token_type(), &token_type);
                self.prev(token);
                Err(error)
            } else {
                Ok(())
            }
        } else {
            Err(ParseError::unexpected_token(
                self.eof_token.line(),
                self.eof_token.token_type(),
                &token_type,
            ))
        }
    }

    fn synchronize(&mut self) {
        // we explicited call next because we push token back after error
        let start_token_type = [
            TokenType::Semicolon, // this one is the end, but we also count
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
            if let Some(token) = self.match_token_type(&start_token_type) {
                // dont fallback if it is semicolon
                if token.token_type() != &TokenType::Semicolon {
                    self.prev(token);
                }
                break;
            } else if self.next().is_none() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {

    use std::ops::{Deref, DerefMut};

    use crate::ast_repr::AstRepr;

    use super::*;

    struct TestParser {
        parser: Parser,
    }

    impl TestParser {
        fn current(&mut self) -> Option<Token> {
            self.parser.next().map(|token| {
                self.parser.prev(token.clone());
                token
            })
        }
    }

    impl From<&str> for TestParser {
        fn from(source: &str) -> Self {
            let mut scanner = Scanner::new(source);
            scanner.scan_tokens();
            assert!(!scanner.had_error());
            TestParser {
                parser: Parser::from(&scanner),
            }
        }
    }

    impl Deref for TestParser {
        type Target = Parser;
        fn deref(&self) -> &Self::Target {
            &self.parser
        }
    }

    impl DerefMut for TestParser {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.parser
        }
    }

    mod test_expression {

        use super::*;

        fn test_parser(sources: &[&str], expected_results: &[ParseResult<&str>]) {
            let mut ast_repr = AstRepr::default();
            for (&src, expected) in std::iter::zip(sources, expected_results) {
                let mut parser = TestParser::from(src);
                let result = parser.expression().map(|expr| ast_repr.expr(&expr));
                assert_eq!(result.as_deref(), expected.as_deref());
            }
        }

        #[test]
        fn consume_without_error() -> ParseResult<()> {
            let source = "(";
            let mut parser = TestParser::from(source);
            parser.consume(TokenType::LeftParen)?;
            assert!(parser.current().is_none());
            Ok(())
        }

        #[test]
        fn consume_with_error() -> ParseResult<()> {
            let source = ")";
            let mut parser = TestParser::from(source);
            let expected_token = TokenType::LeftParen;
            let error = ParseError::unexpected_token(1, &TokenType::RightParen, &expected_token);
            assert_eq!(parser.consume(expected_token), Err(error));
            assert!(parser.current().is_some());
            Ok(())
        }

        #[test]
        fn primary() {
            let sources = [
                "nil",
                "true",
                "false",
                "\"this is string\"",
                "123",
                "123.456",
                "(nil)",
                "(1",
                "variable",
            ];
            let expected_results = [
                Ok("nil"),
                Ok("true"),
                Ok("false"),
                Ok("\"this is string\""),
                Ok("123"),
                Ok("123.456"),
                Ok("Expr::Group(nil)"),
                Err(ParseError::unexpected_token(
                    1,
                    &TokenType::Eof,
                    &TokenType::RightParen,
                )),
                Ok("Expr::Variable(variable)"),
            ];
            test_parser(&sources, &expected_results);
        }

        #[test]
        fn unary() {
            let sources = [
                "-1.2", "-\"a\"", "-nil", "-true", "-false", "!1", "!\"a\"", "!nil", "!true",
                "!false", "-(1.2)", "-x", "!x",
            ];
            let expected_results = [
                Ok("Expr::Unary(- 1.2)"),
                Ok("Expr::Unary(- \"a\")"),
                Ok("Expr::Unary(- nil)"),
                Ok("Expr::Unary(- true)"),
                Ok("Expr::Unary(- false)"),
                Ok("Expr::Unary(! 1)"),
                Ok("Expr::Unary(! \"a\")"),
                Ok("Expr::Unary(! nil)"),
                Ok("Expr::Unary(! true)"),
                Ok("Expr::Unary(! false)"),
                Ok("Expr::Unary(- Expr::Group(1.2))"),
                Ok("Expr::Unary(- Expr::Variable(x))"),
                Ok("Expr::Unary(! Expr::Variable(x))"),
            ];
            test_parser(&sources, &expected_results);
        }

        #[test]
        fn binary() {
            let source = [
                "1+2",
                "3 - 7",
                "true * false",
                "nil / nil",
                "\"a\" == \"b\" ",
                "nil != nil",
                "3 > 7",
                "true >= false",
                "2 < 3",
                "true <= true",
                "x + y",
            ];
            let expected_results = [
                Ok("Expr::Binary(1 + 2)"),
                Ok("Expr::Binary(3 - 7)"),
                Ok("Expr::Binary(true * false)"),
                Ok("Expr::Binary(nil / nil)"),
                Ok("Expr::Binary(\"a\" == \"b\")"),
                Ok("Expr::Binary(nil != nil)"),
                Ok("Expr::Binary(3 > 7)"),
                Ok("Expr::Binary(true >= false)"),
                Ok("Expr::Binary(2 < 3)"),
                Ok("Expr::Binary(true <= true)"),
                Ok("Expr::Binary(Expr::Variable(x) + Expr::Variable(y))"),
            ];
            test_parser(&source, &expected_results);
        }

        #[test]
        fn assignment() {
            let sources = [
                "x = 1;",
                "x = \"string\";",
                "x = true;",
                "x = nil;",
                "x = y;",
                "x = y",
            ];
            let expected_results = [
                Ok("Expr::Assign(x = 1)"),
                Ok("Expr::Assign(x = \"string\")"),
                Ok("Expr::Assign(x = true)"),
                Ok("Expr::Assign(x = nil)"),
                Ok("Expr::Assign(x = Expr::Variable(y))"),
                Ok("Expr::Assign(x = Expr::Variable(y))"),
            ];
            test_parser(&sources, &expected_results);
        }

        #[test]
        fn synchronize_with_semicolon() {
            // synchronize until semicolon, the next token should be `true`.
            let source = "(1 + 2 + 3 nothing; true < false";
            let mut parser = TestParser::from(source);
            let result = parser.expression();
            assert!(result.is_err());
            parser.synchronize();
            assert_eq!(
                result,
                Err(ParseError::unexpected_token(
                    1,
                    &TokenType::Identifier("nothing".to_string()),
                    &TokenType::RightParen
                ))
            );
            assert_eq!(parser.current(), Some(Token::new(TokenType::True, 1)),);
        }

        #[test]
        fn synchronize_without_semicolon() {
            // synchronize until semicolon, the next token should be `return`.
            let source = "(1 + 2 + 3 return true < false";
            let mut parser = TestParser::from(source);
            let result = parser.expression();
            assert!(result.is_err());
            parser.synchronize();
            assert_eq!(
                result,
                Err(ParseError::unexpected_token(
                    1,
                    &TokenType::Identifier("return".to_string()),
                    &TokenType::RightParen
                ))
            );
            assert_eq!(parser.current(), Some(Token::new(TokenType::Return, 1)));
        }
    }

    mod test_statement {

        use super::*;

        fn test_parser(source: &str, expected_statements: &[&str], expected_errors: &[ParseError]) {
            let mut ast_repr = AstRepr::default();
            let mut parser = TestParser::from(source);
            let statements = parser
                .parse()
                .iter()
                .map(|s| ast_repr.stmt(s))
                .collect::<Vec<_>>();
            assert_eq!(statements, expected_statements);
            assert_eq!(parser.errors(), expected_errors);
        }

        #[test]
        fn multiple_expressions_with_errors() {
            let source = "
            \"has semicolon\";
            (\"no right paren\";
            (\"has right paren\");
            \"no semicolon\"";
            let expected_statements = [
                "Stmt::Expr(\"has semicolon\")",
                "Stmt::Expr(Expr::Group(\"has right paren\"))",
            ];
            let expected_errors = [
                ParseError::unexpected_token(3, &TokenType::Semicolon, &TokenType::RightParen),
                ParseError::unexpected_token(5, &TokenType::Eof, &TokenType::Semicolon),
            ];
            test_parser(source, &expected_statements, &expected_errors)
        }

        #[test]
        fn print_expression_with_errors() {
            let source = "
            print \"statement\";
            print \"statement without semicolon\"
            print 1 + 2;";
            let expected_statements = [
                "Stmt::Print(\"statement\")",
                "Stmt::Print(Expr::Binary(1 + 2))",
            ];
            let expected_errors = [ParseError::unexpected_token(
                4,
                &TokenType::Print,
                &TokenType::Semicolon,
            )];
            test_parser(source, &expected_statements, &expected_errors)
        }

        #[test]
        fn variable_declaration_statement() {
            let source = "
            var x = 1; 
            var x = y + 1;
            var x
            print x;
            ";
            let expected_statements = [
                "Stmt::Var(x = 1)",
                "Stmt::Var(x = Expr::Binary(Expr::Variable(y) + 1))",
                "Stmt::Print(Expr::Variable(x))",
            ];
            let expected_errors = [ParseError::unexpected_token(
                5,
                &TokenType::Print,
                &TokenType::Semicolon,
            )];
            test_parser(source, &expected_statements, &expected_errors)
        }

        #[test]
        fn assignment_statement() {
            let source = "
            var x = 1;
            x = 2;
            x = y;
            \"this is not assignment\" = 2
            ";
            let expected_statements = [
                "Stmt::Var(x = 1)",
                "Stmt::Expr(Expr::Assign(x = 2))",
                "Stmt::Expr(Expr::Assign(x = Expr::Variable(y)))",
            ];
            let expected_errors = [ParseError::invalid_assignment(5).without_panic()];
            test_parser(source, &expected_statements, &expected_errors)
        }

        #[test]
        fn assignment_statement_dont_run_to_panic_mode() {
            let source = "
            2 = 1 // this has error
            \"this token should not be eaten\";
            true;
            ";
            let expected_statements = [
                "Stmt::Expr(\"this token should not be eaten\")",
                "Stmt::Expr(true)",
            ];
            let expected_errors = [ParseError::invalid_assignment(2).without_panic()];
            test_parser(source, &expected_statements, &expected_errors)
        }

        #[test]
        fn block_statement() {
            let source = "
            {
                {
                    var x = 1;
                }
                var x = 2;
            }

            {
                1 + 2;
            ";

            let expected_statements =
                ["Stmt::Block(Stmt::Block(Stmt::Var(x = 1)) Stmt::Var(x = 2))"];
            let expected_errors = [ParseError::unexpected_token(
                11,
                &TokenType::Eof,
                &TokenType::RightBrace,
            )];
            test_parser(source, &expected_statements, &expected_errors)
        }
    }
}
