use std::vec::IntoIter;

use crate::{
    error::{LoxError, LoxErrorType},
    expr::{Binary, Expr, Grouping, Literal, Unary},
    scanner::ScanResult,
    token::{Token, TokenType},
};

pub(crate) struct Parser {
    it: IntoIter<Token>,
    buffer: Vec<Token>,
    _errors: Vec<LoxError>,
    _eof_token: Token,
}

impl From<ScanResult> for Parser {
    fn from(value: ScanResult) -> Self {
        Parser::new(value.tokens)
    }
}

impl Parser {
    fn new(mut tokens: Vec<Token>) -> Self {
        let _eof_token = tokens
            .pop()
            .expect("TokenType::Eof must be the end of scan result");
        Parser {
            it: tokens.into_iter(),
            buffer: Vec::with_capacity(16),
            _errors: Vec::new(),
            _eof_token,
        }
    }

    pub fn parse(&mut self) -> Result<Expr, LoxError> {
        let result = self.expresion();
        if result.is_err() {
            dbg!(&result);
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

    fn expresion(&mut self) -> Result<Expr, LoxError> {
        self.equality()
    }

    // @todo: fix this to return token instead,
    fn match_token_type(&mut self, token_type: &[TokenType]) -> bool {
        if let Some(token) = self.next() {
            let contain = token_type.iter().any(|lexeme| lexeme == token.token_type());
            self.prev(token);
            contain
        } else {
            false
        }
    }

    fn equality(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.comparision()?;
        loop {
            if !self.match_token_type(&[TokenType::BangEqual, TokenType::EqualEqual]) {
                break;
            }
            let operator = self.next().unwrap();
            let right = self.comparision()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.term()?;
        loop {
            if !self.match_token_type(&[
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ]) {
                break;
            }
            let operator = self.next().unwrap();
            let right = self.term()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.factor()?;
        loop {
            if !self.match_token_type(&[TokenType::Minus, TokenType::Plus]) {
                break;
            }
            let operator = self.next().unwrap();
            let right = self.factor()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.unary()?;
        loop {
            if !self.match_token_type(&[TokenType::Slash, TokenType::Star]) {
                break;
            }
            let operator = self.next().unwrap();
            let right = self.unary()?;
            expr = Expr::Binary(Binary::new(expr, operator, right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.match_token_type(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.next().unwrap();
            let right = self.unary()?;
            Ok(Expr::Unary(Unary::new(operator, right)))
        } else {
            Ok(self.primary()?)
        }
    }

    fn primary(&mut self) -> Result<Expr, LoxError> {
        if let Some(token) = self.next() {
            match TokenType::from(token.clone()) {
                TokenType::Nil => Ok(Expr::Literal(Literal::Null)),
                TokenType::False => Ok(Expr::Literal(Literal::Bool(false))),
                TokenType::True => Ok(Expr::Literal(Literal::Bool(true))),
                TokenType::Number(number) => Ok(Expr::Literal(Literal::Number(number))),
                TokenType::String(string) => Ok(Expr::Literal(Literal::String(string))),
                TokenType::LeftParen => {
                    let expr = self.expresion()?;
                    self.consume(TokenType::RightParen, ")")?;
                    Ok(Expr::Grouping(Grouping::new(expr)))
                }
                _ => {
                    let error = LoxError::new(
                        token.line(),
                        LoxErrorType::UnexpectedToken(token.lexeme().to_string()),
                    );
                    self.prev(token);
                    Err(error)
                }
            }
        } else {
            unimplemented!("Unexpected eof.")
        }
    }

    fn consume(&mut self, token_type: TokenType, lexeme: &str) -> Result<(), LoxError> {
        if let Some(token) = self.next() {
            if token.token_type() != &token_type {
                let error = LoxError::new(
                    token.line(),
                    LoxErrorType::ParserExpectToken(token.lexeme().to_string(), lexeme.to_string()),
                );
                self.prev(token);
                Err(error)
            } else {
                Ok(())
            }
        } else {
            unimplemented!("Unexpected eof.")
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
            if self.match_token_type(&start_token_type) || self.next().is_none() {
                break;
            }
        }
    }
}
