//! Token 定义 —— Mini 语言词法分析器使用的词法单元。
//!
//! 每个 Token 携带类型和源码位置（行号、列号），用于错误报告。

/// 词法分析器产生的所有 Token 类型。
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // --- 字面量 ---
    Int(i64),
    Float(f64),
    Str(String),
    Ident(String),

    // --- 关键字 ---
    Let,
    Const,
    Fn,
    If,
    Else,
    While,
    For,
    In,
    Return,
    True,
    False,
    Break,
    Continue,
    Null,

    // --- 运算符 ---
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    PlusAssign,
    MinusAssign,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Bang,

    // --- 分隔符 ---
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Arrow,

    Eof,
}

/// 单个 Token，附带源码位置。
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Token { kind, line, col }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Int(v) => write!(f, "{}", v),
            TokenKind::Float(v) => write!(f, "{}", v),
            TokenKind::Str(s) => write!(f, "\"{}\"", s),
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Assign => write!(f, "="),
            TokenKind::PlusAssign => write!(f, "+="),
            TokenKind::MinusAssign => write!(f, "-="),
            TokenKind::Eq => write!(f, "=="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::And => write!(f, "&&"),
            TokenKind::Or => write!(f, "||"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Eof => write!(f, "<eof>"),
        }
    }
}

/// 判断一个标识符字符串是否为保留关键字。
pub fn keyword_lookup(ident: &str) -> Option<TokenKind> {
    match ident {
        "let" => Some(TokenKind::Let),
        "const" => Some(TokenKind::Const),
        "fn" => Some(TokenKind::Fn),
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "while" => Some(TokenKind::While),
        "for" => Some(TokenKind::For),
        "in" => Some(TokenKind::In),
        "return" => Some(TokenKind::Return),
        "true" => Some(TokenKind::True),
        "false" => Some(TokenKind::False),
        "break" => Some(TokenKind::Break),
        "continue" => Some(TokenKind::Continue),
        "null" => Some(TokenKind::Null),
        _ => None,
    }
}
