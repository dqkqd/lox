use std::collections::HashMap;

use crate::{
    error::{LoxError, LoxErrorType},
    token::{Number, Token, TokenType},
};

// alpha for identifier
fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

pub(crate) fn generate_static_reserved_keywords() -> HashMap<String, TokenType> {
    let mut keywords = HashMap::new();
    keywords.insert(String::from("and"), TokenType::And);
    keywords.insert(String::from("class"), TokenType::Class);
    keywords.insert(String::from("else"), TokenType::Else);
    keywords.insert(String::from("false"), TokenType::False);
    keywords.insert(String::from("for"), TokenType::For);
    keywords.insert(String::from("fun"), TokenType::Fun);
    keywords.insert(String::from("if"), TokenType::If);
    keywords.insert(String::from("nil"), TokenType::Nil);
    keywords.insert(String::from("or"), TokenType::Or);
    keywords.insert(String::from("print"), TokenType::Print);
    keywords.insert(String::from("return"), TokenType::Return);
    keywords.insert(String::from("super"), TokenType::Super);
    keywords.insert(String::from("this"), TokenType::This);
    keywords.insert(String::from("true"), TokenType::True);
    keywords.insert(String::from("var"), TokenType::Var);
    keywords.insert(String::from("while"), TokenType::While);
    keywords
}

#[derive(Default)]
pub(crate) struct ScanResult {
    pub tokens: Vec<Token>,
    errors: Vec<LoxError>,
}

impl ScanResult {
    fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn add_error(&mut self, error: LoxError) {
        self.errors.push(error)
    }

    pub fn update(&mut self, result: Result<Token, LoxError>) {
        match result {
            Ok(token) => self.add_token(token),
            Err(error) => self.add_error(error),
        }
    }

    pub fn errors(&self) -> &[LoxError] {
        self.errors.as_ref()
    }

    pub fn had_error(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[derive(Debug)]
pub(crate) struct Scanner<'a> {
    it: std::str::Chars<'a>,
    line: usize,
    buffer: Vec<char>,
    reserved_keywords: HashMap<String, TokenType>,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        Scanner {
            it: source.chars(),
            line: 1,
            buffer: Vec::with_capacity(16),
            reserved_keywords: generate_static_reserved_keywords(),
        }
    }

    fn prev(&mut self, c: char) {
        self.buffer.push(c);
    }

    fn next(&mut self) -> Option<char> {
        if self.buffer.is_empty() {
            self.it.next()
        } else {
            self.buffer.pop()
        }
    }

    fn read_while<F>(&mut self, f: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut string = String::new();
        while let Some(c) = self.next() {
            if !f(c) {
                self.prev(c);
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

    fn string(&mut self) -> Result<Token, LoxError> {
        let string = self.read_while(|c| c != '"');
        match self.next() {
            Some(_) => Ok(self.make_token(&string, TokenType::String(string.clone()))),
            None => Err(LoxError::new(self.line, LoxErrorType::UnterminatedString)),
        }
    }

    fn number(&mut self) -> Token {
        let mut numstr = self.read_while(|c| c.is_ascii_digit());
        match self.next() {
            Some('.') => {
                let decimal = self.read_while(|c| c.is_ascii_digit());
                if decimal.is_empty() {
                    self.prev('.');
                } else {
                    numstr.push('.');
                    numstr.push_str(&decimal);
                }
            }
            Some(c) => self.prev(c),
            None => (),
        }

        // this is always success
        let number = numstr.parse::<Number>().unwrap();
        self.make_token(&numstr, TokenType::Number(number))
    }

    fn identifier(&mut self) -> Token {
        let identifier = self.read_while(|c| c.is_ascii_alphanumeric());
        match self.reserved_keywords.get(&identifier) {
            Some(token_type) => self.make_token(&identifier, token_type.clone()),
            None => self.make_token(&identifier, TokenType::Identifier(identifier.clone())),
        }
    }

    fn make_token(&self, lexeme: &str, token_type: TokenType) -> Token {
        Token::new(token_type, lexeme, self.line)
    }

    fn scan_token(&mut self, c: char) -> Option<Result<Token, LoxError>> {
        let token = match c {
            // single lexeme
            '(' => self.make_token("(", TokenType::LeftParen),
            ')' => self.make_token(")", TokenType::RightParen),
            '{' => self.make_token("{", TokenType::LeftBrace),
            '}' => self.make_token("}", TokenType::RightBrace),
            ',' => self.make_token(",", TokenType::Comma),
            '.' => self.make_token(".", TokenType::Dot),
            '-' => self.make_token("-", TokenType::Minus),
            '+' => self.make_token("+", TokenType::Plus),
            ';' => self.make_token(";", TokenType::Semicolon),
            '*' => self.make_token("*", TokenType::Star),

            // operators
            '!' => match self.next() {
                Some('=') => self.make_token("!=", TokenType::BangEqual),
                c => {
                    if let Some(c) = c {
                        self.prev(c);
                    }
                    self.make_token("!", TokenType::Bang)
                }
            },
            '=' => match self.next() {
                Some('=') => self.make_token("==", TokenType::EqualEqual),
                c => {
                    if let Some(c) = c {
                        self.prev(c);
                    }
                    self.make_token("=", TokenType::Equal)
                }
            },
            '<' => match self.next() {
                Some('=') => self.make_token("<=", TokenType::LessEqual),
                c => {
                    if let Some(c) = c {
                        self.prev(c)
                    }
                    self.make_token("<", TokenType::Less)
                }
            },
            '>' => match self.next() {
                Some('=') => self.make_token(">=", TokenType::GreaterEqual),
                c => {
                    if let Some(c) = c {
                        self.prev(c)
                    }
                    self.make_token(">", TokenType::Greater)
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
                    if let Some(c) = c {
                        self.prev(c)
                    }
                    self.make_token("/", TokenType::Slash)
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
                    self.prev(c);
                    self.number()
                }
                false => match is_alpha(c) {
                    true => {
                        // identifier
                        self.prev(c);
                        self.identifier()
                    }
                    false => {
                        return Some(Err(LoxError::new(
                            self.line,
                            LoxErrorType::UnexpectedCharacter(c),
                        )))
                    }
                },
            },
        };
        Some(Ok(token))
    }

    fn scan_tokens(&mut self) -> ScanResult {
        let mut scan_result = ScanResult::default();
        while let Some(c) = self.next() {
            if let Some(r) = self.scan_token(c) {
                scan_result.update(r);
            }
        }

        let eof = self.make_token("", TokenType::Eof);
        scan_result.add_token(eof);

        scan_result
    }
}

pub(crate) fn scan(source: &str) -> ScanResult {
    Scanner::new(source).scan_tokens()
}

#[cfg(test)]
mod test {

    use super::*;

    fn check(source: &str, expected_tokens: &[Token], expected_error: &[LoxError]) {
        let scan_result = Scanner::new(source).scan_tokens();
        assert_eq!(scan_result.tokens, expected_tokens);
        assert_eq!(scan_result.errors, expected_error);
    }

    #[test]
    fn scan_single_lexeme() {
        let source = "()
        {},.-+
        ;*";
        let tokens = [
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::LeftBrace, "{", 2),
            Token::new(TokenType::RightBrace, "}", 2),
            Token::new(TokenType::Comma, ",", 2),
            Token::new(TokenType::Dot, ".", 2),
            Token::new(TokenType::Minus, "-", 2),
            Token::new(TokenType::Plus, "+", 2),
            Token::new(TokenType::Semicolon, ";", 3),
            Token::new(TokenType::Star, "*", 3),
            Token::new(TokenType::Eof, "", 3),
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
            Token::new(TokenType::BangEqual, "!=", 1),
            Token::new(TokenType::Bang, "!", 1),
            Token::new(TokenType::EqualEqual, "==", 2),
            Token::new(TokenType::Equal, "=", 2),
            Token::new(TokenType::LessEqual, "<=", 3),
            Token::new(TokenType::Less, "<", 3),
            Token::new(TokenType::GreaterEqual, ">=", 4),
            Token::new(TokenType::Greater, ">", 4),
            Token::new(TokenType::Eof, "", 4),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_comments() {
        let source = "// first comment
        // second comment
        // third comment";
        let tokens = [Token::new(TokenType::Eof, "", 3)];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_string() {
        let source = "\"first string\"
        \"second string\"
        ";
        let tokens = [
            Token::new(
                TokenType::String("first string".to_string()),
                "first string",
                1,
            ),
            Token::new(
                TokenType::String("second string".to_string()),
                "second string",
                2,
            ),
            Token::new(TokenType::Eof, "", 3),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_string_with_error() {
        let source = "\"unterminated string";
        let tokens = [Token::new(TokenType::Eof, "", 1)];
        let errors = [LoxError::new(1, LoxErrorType::UnterminatedString)];
        check(source, &tokens, &errors);
    }

    #[test]
    fn scan_decimal_number() {
        let source = "123.456";
        let tokens = [
            Token::new(TokenType::Number(123.456), "123.456", 1),
            Token::new(TokenType::Eof, "", 1),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_integral_number() {
        let source = "123";
        let tokens = [
            Token::new(TokenType::Number(123.0), "123", 1),
            Token::new(TokenType::Eof, "", 1),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_number_without_dot() {
        let source = "123.";
        let tokens = [
            Token::new(TokenType::Number(123.0), "123", 1),
            Token::new(TokenType::Dot, ".", 1),
            Token::new(TokenType::Eof, "", 1),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_identifier() {
        let source = "var language = \"lox\"";
        let tokens = [
            Token::new(TokenType::Var, "var", 1),
            Token::new(TokenType::Identifier("language".to_string()), "language", 1),
            Token::new(TokenType::Equal, "=", 1),
            Token::new(TokenType::String("lox".to_string()), "lox", 1),
            Token::new(TokenType::Eof, "", 1),
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
            Token::new(TokenType::And, "and", 1),
            Token::new(TokenType::Class, "class", 1),
            Token::new(TokenType::Else, "else", 1),
            Token::new(TokenType::False, "false", 2),
            Token::new(TokenType::For, "for", 2),
            Token::new(TokenType::Fun, "fun", 2),
            Token::new(TokenType::If, "if", 3),
            Token::new(TokenType::Nil, "nil", 3),
            Token::new(TokenType::Or, "or", 3),
            Token::new(TokenType::Print, "print", 3),
            Token::new(TokenType::Return, "return", 4),
            Token::new(TokenType::Super, "super", 4),
            Token::new(TokenType::This, "this", 4),
            Token::new(TokenType::True, "true", 5),
            Token::new(TokenType::Var, "var", 5),
            Token::new(TokenType::While, "while", 5),
            Token::new(TokenType::Eof, "", 5),
        ];
        check(source, &tokens, &[]);
    }

    #[test]
    fn scan_unexpected_character() {
        let source = "@#";
        let tokens = [Token::new(TokenType::Eof, "", 1)];
        let errors = [
            LoxError::new(1, LoxErrorType::UnexpectedCharacter('@')),
            LoxError::new(1, LoxErrorType::UnexpectedCharacter('#')),
        ];
        check(source, &tokens, &errors);
    }
}
