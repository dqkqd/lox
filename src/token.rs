use crate::{object::Number, source::CharPos};

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier(String),
    String(String),
    Number(Number),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl From<Token> for TokenType {
    fn from(value: Token) -> Self {
        value.token_type
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) struct Token {
    token_type: TokenType,
    lexeme: String,
    start_pos: CharPos,
    end_pos: CharPos,
}

impl Token {
    pub fn new(token_type: TokenType, start_pos: CharPos, end_pos: CharPos) -> Self {
        let lexeme = token_type.to_string();
        Self {
            token_type,
            lexeme,
            start_pos,
            end_pos,
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn lexeme(&self) -> &str {
        self.lexeme.as_ref()
    }

    pub fn line(&self) -> usize {
        self.start_pos.line
    }
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            TokenType::LeftParen => "(".to_string(),
            TokenType::RightParen => ")".to_string(),
            TokenType::LeftBrace => "{".to_string(),
            TokenType::RightBrace => "}".to_string(),
            TokenType::Comma => ",".to_string(),
            TokenType::Dot => ".".to_string(),
            TokenType::Minus => "-".to_string(),
            TokenType::Plus => "+".to_string(),
            TokenType::Semicolon => ";".to_string(),
            TokenType::Slash => "/".to_string(),
            TokenType::Star => "*".to_string(),
            TokenType::Bang => "!".to_string(),
            TokenType::BangEqual => "!=".to_string(),
            TokenType::Equal => "=".to_string(),
            TokenType::EqualEqual => "==".to_string(),
            TokenType::Greater => ">".to_string(),
            TokenType::GreaterEqual => ">=".to_string(),
            TokenType::Less => "<".to_string(),
            TokenType::LessEqual => "<=".to_string(),
            TokenType::Identifier(s) => s.to_string(),
            TokenType::String(s) => s.to_string(),
            TokenType::Number(n) => n.to_string(),
            TokenType::And => "and".to_string(),
            TokenType::Class => "class".to_string(),
            TokenType::Else => "else".to_string(),
            TokenType::False => "false".to_string(),
            TokenType::Fun => "fun".to_string(),
            TokenType::For => "for".to_string(),
            TokenType::If => "if".to_string(),
            TokenType::Nil => "nil".to_string(),
            TokenType::Or => "or".to_string(),
            TokenType::Print => "print".to_string(),
            TokenType::Return => "return".to_string(),
            TokenType::Super => "super".to_string(),
            TokenType::This => "this".to_string(),
            TokenType::True => "true".to_string(),
            TokenType::Var => "var".to_string(),
            TokenType::While => "while".to_string(),
            TokenType::Eof => "EOF".to_string(),
        }
    }
}
