use std::collections::HashMap;

use crate::{
    error::syntax_error::SyntaxError,
    object::Number,
    token::{Token, TokenType},
};

// alpha for identifier
fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

pub(crate) fn generate_static_reserved_keywords() -> HashMap<String, TokenType> {
    let mut keywords = HashMap::new();
    let reserved_token = [
        TokenType::And,
        TokenType::Class,
        TokenType::Else,
        TokenType::False,
        TokenType::For,
        TokenType::Fun,
        TokenType::If,
        TokenType::Nil,
        TokenType::Or,
        TokenType::Print,
        TokenType::Return,
        TokenType::Super,
        TokenType::This,
        TokenType::True,
        TokenType::Var,
        TokenType::While,
    ];
    for token_type in reserved_token {
        keywords.insert(token_type.to_string(), token_type);
    }
    keywords
}

type ScanResult<T> = Result<T, SyntaxError>;

#[derive(Debug)]
pub(crate) struct Scanner {
    source: Vec<char>,
    line: usize,
    current: usize,
    reserved_keywords: HashMap<String, TokenType>,
    tokens: Vec<Token>,
    errors: Vec<SyntaxError>,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner {
            source: source.chars().collect(),
            line: 1,
            current: 0,
            reserved_keywords: generate_static_reserved_keywords(),
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn errors(&self) -> &[SyntaxError] {
        &self.errors
    }

    pub fn had_error(&self) -> bool {
        !self.errors.is_empty()
    }

    fn prev(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }

    fn next(&mut self) -> Option<char> {
        if self.current >= self.source.len() {
            None
        } else {
            self.current += 1;
            Some(self.source[self.current - 1])
        }
    }

    fn read_while<F>(&mut self, f: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut string = String::new();
        while let Some(c) = self.next() {
            if !f(c) {
                self.prev();
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            string.push(c);
        }
        string
    }

    fn single_line_comment(&mut self) -> String {
        self.read_while(|c| c != '\n')
    }

    fn string(&mut self) -> ScanResult<Token> {
        let string = self.read_while(|c| c != '"');
        match self.next() {
            Some(_) => Ok(self.make_token(TokenType::String(string))),
            None => Err(SyntaxError::unterminated_string(self.line)),
        }
    }

    fn number(&mut self) -> Token {
        let mut numstr = self.read_while(|c| c.is_ascii_digit());
        match self.next() {
            Some('.') => {
                let decimal = self.read_while(|c| c.is_ascii_digit());
                if decimal.is_empty() {
                    self.prev();
                } else {
                    numstr.push('.');
                    numstr.push_str(&decimal);
                }
            }
            Some(_) => self.prev(),
            None => (),
        }

        // this is always success
        let number = numstr.parse::<Number>().unwrap();
        self.make_token(TokenType::Number(number))
    }

    fn identifier(&mut self) -> Token {
        let identifier = self.read_while(|c| c.is_ascii_alphanumeric());
        match self.reserved_keywords.get(&identifier) {
            Some(token_type) => self.make_token(token_type.clone()),
            None => self.make_token(TokenType::Identifier(identifier.clone())),
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token::new(token_type, self.line)
    }

    fn scan_token(&mut self, c: char) -> Option<ScanResult<Token>> {
        let token = match c {
            // single lexeme
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            ';' => self.make_token(TokenType::Semicolon),
            '*' => self.make_token(TokenType::Star),

            // operators
            '!' => match self.next() {
                Some('=') => self.make_token(TokenType::BangEqual),
                c => {
                    if c.is_some() {
                        self.prev();
                    }
                    self.make_token(TokenType::Bang)
                }
            },
            '=' => match self.next() {
                Some('=') => self.make_token(TokenType::EqualEqual),
                c => {
                    if c.is_some() {
                        self.prev();
                    }
                    self.make_token(TokenType::Equal)
                }
            },
            '<' => match self.next() {
                Some('=') => self.make_token(TokenType::LessEqual),
                c => {
                    if c.is_some() {
                        self.prev()
                    }
                    self.make_token(TokenType::Less)
                }
            },
            '>' => match self.next() {
                Some('=') => self.make_token(TokenType::GreaterEqual),
                c => {
                    if c.is_some() {
                        self.prev()
                    }
                    self.make_token(TokenType::Greater)
                }
            },

            // comment.
            // @TODO: add comment type /* */
            '/' => match self.next() {
                Some('/') => {
                    // read until next line
                    self.single_line_comment();
                    return None;
                }
                c => {
                    if c.is_some() {
                        self.prev()
                    }
                    self.make_token(TokenType::Slash)
                }
            },

            // string
            '"' => {
                return Some(self.string());
            }

            // whitespace
            '\n' | ' ' | '\r' | '\t' => {
                if c == '\n' {
                    self.line += 1;
                }
                return None;
            }

            c => match c.is_ascii_digit() {
                true => {
                    // number
                    self.prev();
                    self.number()
                }
                false => match is_alpha(c) {
                    true => {
                        // identifier
                        self.prev();
                        self.identifier()
                    }
                    false => {
                        return Some(Err(SyntaxError::unexpected_character(self.line, c)));
                    }
                },
            },
        };
        Some(Ok(token))
    }

    pub fn scan_tokens(&mut self) {
        while let Some(c) = self.next() {
            if let Some(result) = self.scan_token(c) {
                match result {
                    Ok(token) => self.tokens.push(token),
                    Err(err) => self.errors.push(err),
                }
            }
        }

        let eof = self.make_token(TokenType::Eof);
        self.tokens.push(eof);
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn check(source: &str, expected_tokens: &[Token], expected_error: &[SyntaxError]) {
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();
        assert_eq!(scanner.tokens, expected_tokens);
        assert_eq!(scanner.errors, expected_error);
    }

    #[test]
    fn scan_single_lexeme() {
        let source = "()
        {},.-+
        ;*";
        let tokens = [
            Token::new(TokenType::LeftParen, 1),
            Token::new(TokenType::RightParen, 1),
            Token::new(TokenType::LeftBrace, 2),
            Token::new(TokenType::RightBrace, 2),
            Token::new(TokenType::Comma, 2),
            Token::new(TokenType::Dot, 2),
            Token::new(TokenType::Minus, 2),
            Token::new(TokenType::Plus, 2),
            Token::new(TokenType::Semicolon, 3),
            Token::new(TokenType::Star, 3),
            Token::new(TokenType::Eof, 3),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_operators() {
        let source = "!= !
         == = 
         <= < 
         >= >";
        let tokens = [
            Token::new(TokenType::BangEqual, 1),
            Token::new(TokenType::Bang, 1),
            Token::new(TokenType::EqualEqual, 2),
            Token::new(TokenType::Equal, 2),
            Token::new(TokenType::LessEqual, 3),
            Token::new(TokenType::Less, 3),
            Token::new(TokenType::GreaterEqual, 4),
            Token::new(TokenType::Greater, 4),
            Token::new(TokenType::Eof, 4),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_comments() {
        let source = "// first comment
        // second comment
        // third comment";
        let tokens = [Token::new(TokenType::Eof, 3)];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_string() {
        let source = "\"first string\"
        \"second string\"
        ";
        let tokens = [
            Token::new(TokenType::String("first string".to_string()), 1),
            Token::new(TokenType::String("second string".to_string()), 2),
            Token::new(TokenType::Eof, 3),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_string_with_error() {
        let source = "\"unterminated string";
        let tokens = [Token::new(TokenType::Eof, 1)];
        let errors = [SyntaxError::unterminated_string(1)];
        check(source, &tokens, &errors);
    }

    #[test]
    fn scan_decimal_number() {
        let source = "123.456";
        let tokens = [
            Token::new(TokenType::Number(123.456), 1),
            Token::new(TokenType::Eof, 1),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_integral_number() {
        let source = "123";
        let tokens = [
            Token::new(TokenType::Number(123.0), 1),
            Token::new(TokenType::Eof, 1),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_number_without_dot() {
        let source = "123.";
        let tokens = [
            Token::new(TokenType::Number(123.0), 1),
            Token::new(TokenType::Dot, 1),
            Token::new(TokenType::Eof, 1),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_identifier() {
        let source = "var language = \"lox\"";
        let tokens = [
            Token::new(TokenType::Var, 1),
            Token::new(TokenType::Identifier("language".to_string()), 1),
            Token::new(TokenType::Equal, 1),
            Token::new(TokenType::String("lox".to_string()), 1),
            Token::new(TokenType::Eof, 1),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_reserved_keywords() {
        let source = "and class else
        false for fun
        if nil or print
        return super this 
        true var while";
        let tokens = [
            Token::new(TokenType::And, 1),
            Token::new(TokenType::Class, 1),
            Token::new(TokenType::Else, 1),
            Token::new(TokenType::False, 2),
            Token::new(TokenType::For, 2),
            Token::new(TokenType::Fun, 2),
            Token::new(TokenType::If, 3),
            Token::new(TokenType::Nil, 3),
            Token::new(TokenType::Or, 3),
            Token::new(TokenType::Print, 3),
            Token::new(TokenType::Return, 4),
            Token::new(TokenType::Super, 4),
            Token::new(TokenType::This, 4),
            Token::new(TokenType::True, 5),
            Token::new(TokenType::Var, 5),
            Token::new(TokenType::While, 5),
            Token::new(TokenType::Eof, 5),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_unexpected_character() {
        let source = "@#";
        let tokens = [Token::new(TokenType::Eof, 1)];
        let errors = [
            SyntaxError::unexpected_character(1, '@'),
            SyntaxError::unexpected_character(1, '#'),
        ];
        check(source, &tokens, &errors);
    }
}
