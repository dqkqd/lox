use std::vec::IntoIter;

use crate::{
    error::parse_error::ParseError,
    expr::{Binary, Expr, Grouping, Unary},
    object::Object,
    scanner::Scanner,
    stmt::Stmt,
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

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        loop {
            if self.is_end() {
                break;
            }

            match self.statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    self.synchronize();
                    self.errors.push(err)
                }
            }
        }
        statements
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

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.match_token_type(&[TokenType::Print]).is_some() {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon)?;
        Ok(Stmt::Expression(expr))
    }

    // @todo this method currently pub, move this to private after all stmts are added
    pub fn expression(&mut self) -> ParseResult<Expr> {
        self.equality()
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
            Err(ParseError::expected_expression(self.eof_token.line()))
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
        ];
        let expected_results = [
            Ok("nil"),
            Ok("true"),
            Ok("false"),
            Ok("\"this is string\""),
            Ok("123"),
            Ok("123.456"),
            Ok("Expr::Group(nil)"),
            Err(ParseError::expected_expression(1)),
        ];
        test_parser(&sources, &expected_results);
    }

    #[test]
    fn unary() {
        let sources = [
            "-1.2", "-\"a\"", "-nil", "-true", "-false", "!1", "!\"a\"", "!nil", "!true", "!false",
            "-(1.2)",
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
        ];
        test_parser(&source, &expected_results);
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
        assert_eq!(parser.current(), Some(Token::new(TokenType::Return, 1)),);
    }
}
