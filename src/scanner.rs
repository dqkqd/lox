use std::collections::HashMap;

use crate::{
    error::{reporter::ErrorReporter, syntax_error::SyntaxError},
    object::Number,
    source::{CharPos, SourcePos},
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
    source_pos: SourcePos,
    line: usize,
    current: usize,
    reserved_keywords: HashMap<String, TokenType>,
    tokens: Vec<Token>,
    errors: Vec<SyntaxError>,
}

impl ErrorReporter<SyntaxError> for Scanner {
    fn errors(&self) -> &[SyntaxError] {
        &self.errors
    }
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        let source_pos = SourcePos::new(source);
        Scanner {
            source: source.chars().collect(),
            source_pos,
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

    fn prev(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }

    fn prev_pos(&self) -> Option<CharPos> {
        if self.current > 0 {
            self.source_pos.get(self.current - 1)
        } else {
            None
        }
    }

    fn peek(&self) -> Option<char> {
        if self.current >= self.source.len() {
            None
        } else {
            Some(self.source[self.current])
        }
    }

    fn next(&mut self) -> Option<char> {
        self.peek().map(|c| {
            self.current += 1;
            c
        })
    }

    fn lookahead(&self, distance: usize) -> Option<char> {
        let index = self.current + distance;
        if index >= self.source.len() {
            None
        } else {
            Some(self.source[index])
        }
    }

    fn new_line(&mut self) {
        self.line += 1;
    }

    fn read_while<F>(&mut self, f: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut string = String::new();
        while let Some(c) = self.peek() {
            if !f(c) {
                break;
            }
            self.next().map(|c| {
                if c == '\n' {
                    self.new_line();
                }
                string.push(c);
            });
        }
        string
    }

    fn single_line_comment(&mut self) -> String {
        self.read_while(|c| c != '\n')
    }

    fn string(&mut self) -> ScanResult<TokenType> {
        let string = self.read_while(|c| c != '"');
        match self.next() {
            Some(_) => Ok(TokenType::String(string)),
            None => Err(SyntaxError::unterminated_string(self.prev_pos().unwrap())),
        }
    }

    fn number(&mut self) -> TokenType {
        let mut numstr = self.read_while(|c| c.is_ascii_digit());
        if let Some('.') = self.peek() {
            let has_digit = self
                .lookahead(1)
                .map(|c| c.is_ascii_digit())
                .unwrap_or_default();
            if has_digit {
                // skip dot
                self.next();
                numstr.push('.');
                let fraction = self.read_while(|c| c.is_ascii_digit());
                numstr.push_str(&fraction);
            }
        }

        // this is always success
        let number = numstr.parse::<Number>().unwrap();
        TokenType::Number(number)
    }

    fn identifier(&mut self) -> TokenType {
        let identifier = self.read_while(|c| c.is_ascii_alphanumeric());
        match self.reserved_keywords.get(&identifier) {
            Some(token_type) => token_type.clone(),
            None => TokenType::Identifier(identifier.clone()),
        }
    }

    fn make_token(&self, token_type: TokenType, prev_pos: CharPos, cur_pos: CharPos) -> Token {
        Token::new(token_type, prev_pos, cur_pos)
    }

    fn scan_token(&mut self, c: char) -> Option<ScanResult<Token>> {
        let prev_pos = self.prev_pos().unwrap();

        let token_type = match c {
            // single lexeme
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '-' => TokenType::Minus,
            '+' => TokenType::Plus,
            ';' => TokenType::Semicolon,
            '*' => TokenType::Star,

            // operators
            '!' => match self.peek() {
                Some('=') => {
                    self.next();

                    TokenType::BangEqual
                }
                _ => TokenType::Bang,
            },
            '=' => match self.peek() {
                Some('=') => {
                    self.next();

                    TokenType::EqualEqual
                }
                _ => TokenType::Equal,
            },
            '<' => match self.peek() {
                Some('=') => {
                    self.next();

                    TokenType::LessEqual
                }
                _ => TokenType::Less,
            },
            '>' => match self.peek() {
                Some('=') => {
                    self.next();

                    TokenType::GreaterEqual
                }
                _ => TokenType::Greater,
            },

            // comment.
            // @TODO: add comment type /* */
            '/' => match self.peek() {
                Some('/') => {
                    // read until next line
                    self.next();
                    self.single_line_comment();
                    return None;
                }
                _ => TokenType::Slash,
            },

            // string
            '"' => {
                let string = self.string();
                match string {
                    Err(err) => return Some(Err(err)),
                    Ok(s) => s,
                }
            }

            // whitespace
            '\n' | ' ' | '\r' | '\t' => {
                if c == '\n' {
                    self.new_line();
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
                        return Some(Err(SyntaxError::unexpected_character(
                            self.prev_pos().unwrap(),
                        )));
                    }
                },
            },
        };

        let cur_pos = self.prev_pos().unwrap();
        let token = self.make_token(token_type, prev_pos, cur_pos);

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

        let eof = self.make_token(
            TokenType::Eof,
            self.prev_pos().unwrap(),
            self.prev_pos().unwrap(),
        );
        self.tokens.push(eof);
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::error::reporter::Reporter;

    use std::io::Write;

    fn test_scanner(source: &str, expected_output: &str) -> Result<(), std::io::Error> {
        let mut result = Vec::new();
        let mut scanner = Scanner::new(source);

        scanner.scan_tokens();

        for token in scanner.tokens() {
            writeln!(
                &mut result,
                "line: {}, token: {}",
                token.start_pos().line + 1,
                token.lexeme()
            )?;
        }

        let source_pos = SourcePos::new(source);
        let reporter = Reporter::new(&source_pos);
        writeln!(&mut result, "{}", scanner.error_msg(&reporter))?;

        let result = String::from_utf8(result).unwrap();
        assert_eq!(result.trim(), expected_output.trim());
        Ok(())
    }

    #[test]
    fn scan_single_lexeme() -> Result<(), std::io::Error> {
        let source = r#"()
{},.-+
;*"#;
        let expected_output = r#"
line: 1, token: (
line: 1, token: )
line: 2, token: {
line: 2, token: }
line: 2, token: ,
line: 2, token: .
line: 2, token: -
line: 2, token: +
line: 3, token: ;
line: 3, token: *
line: 3, token: EOF"#;
        test_scanner(source, expected_output)
    }

    #[test]
    fn scan_operators() -> Result<(), std::io::Error> {
        let source = r#"!= !
== = 
<= < 
>= >"#;
        let expected_output = r#"
line: 1, token: !=
line: 1, token: !
line: 2, token: ==
line: 2, token: =
line: 3, token: <=
line: 3, token: <
line: 4, token: >=
line: 4, token: >
line: 4, token: EOF
"#;
        test_scanner(source, expected_output)
    }

    #[test]
    fn scan_comments() -> Result<(), std::io::Error> {
        let source = r#"
// first comment
// second comment
// third comment
"#;
        let expected_output = r#"line: 4, token: EOF"#;
        test_scanner(source, expected_output)
    }

    #[test]
    fn scan_string() -> Result<(), std::io::Error> {
        let source = r#"
"first string"
"second string"
"#;
        let expected_output = r#"
line: 2, token: first string
line: 3, token: second string
line: 3, token: EOF
"#;
        test_scanner(source, expected_output)
    }

    #[test]
    fn scan_string_with_error() -> Result<(), std::io::Error> {
        let source = r#""unterminated string"#;
        let expected_output = r#"
line: 1, token: EOF
[line 1]: SyntaxError: Unterminated string
"unterminated string
                   ^
"#;
        test_scanner(source, expected_output)
    }

    #[test]
    fn scan_decimal_number() -> Result<(), std::io::Error> {
        let source = r#"123.456"#;
        let expected_output = r#"
line: 1, token: 123.456
line: 1, token: EOF
"#;
        test_scanner(source, expected_output)
    }

    #[test]
    fn scan_integral_number() -> Result<(), std::io::Error> {
        let source = r#"123"#;
        let expected_output = r#"
line: 1, token: 123
line: 1, token: EOF
"#;
        test_scanner(source, expected_output)
    }

    #[test]
    fn scan_number_without_dot() -> Result<(), std::io::Error> {
        let source = r#"123."#;
        let expected_output = r#"
line: 1, token: 123
line: 1, token: .
line: 1, token: EOF
"#;
        test_scanner(source, expected_output)
    }

    #[test]
    fn scan_identifier() -> Result<(), std::io::Error> {
        let source = r#"var language = "lox""#;
        let expected_output = r#"
line: 1, token: var
line: 1, token: language
line: 1, token: =
line: 1, token: lox
line: 1, token: EOF
"#;
        test_scanner(source, expected_output)
    }

    #[test]
    fn scan_reserved_keywords() -> Result<(), std::io::Error> {
        let source = r#"and class else
false for fun
if nil or print
return super this 
true var while
"#;
        let expected_output = r#"
line: 1, token: and
line: 1, token: class
line: 1, token: else
line: 2, token: false
line: 2, token: for
line: 2, token: fun
line: 3, token: if
line: 3, token: nil
line: 3, token: or
line: 3, token: print
line: 4, token: return
line: 4, token: super
line: 4, token: this
line: 5, token: true
line: 5, token: var
line: 5, token: while
line: 5, token: EOF
"#;
        test_scanner(source, expected_output)
    }

    #[test]
    fn scan_unexpected_character() -> Result<(), std::io::Error> {
        let source = r#"@#"#;
        let expected_output = r#"
line: 1, token: EOF
[line 1]: SyntaxError: Unexpected character `@`
@#
^
[line 1]: SyntaxError: Unexpected character `#`
@#
 ^
"#;
        test_scanner(source, expected_output)
    }
}
