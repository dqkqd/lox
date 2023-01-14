use std::vec::IntoIter;

use crate::{
    error::parse_error::ParseError,
    expr::{Binary, Expr, Grouping, Unary},
    object::Object,
    scanner::Scanner,
    token::{Token, TokenType},
};

type ParseResult<T> = Result<T, ParseError>;

pub(crate) struct Parser {
    it: IntoIter<Token>,
    buffer: Vec<Token>,
    _errors: Vec<ParseError>,
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
            _errors: Vec::new(),
            eof_token: _eof_token,
        }
    }

    pub fn parse(&mut self) -> ParseResult<Expr> {
        let result = self.expresion();
        if result.is_err() {
            self.synchronize();
        }
        result
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

    fn expresion(&mut self) -> ParseResult<Expr> {
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
                    let expr = self.expresion()?;
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

    fn consume(&mut self, token_type: TokenType) -> Result<(), ParseError> {
        if let Some(token) = self.next() {
            if token.token_type() != &token_type {
                let error = ParseError::unexpected_token(token.line(), token.lexeme());
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
                self.prev(token);
                break;
            } else if self.next().is_none() {
                break;
            }
        }
    }
}
