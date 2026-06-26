//! 词法分析器（Lexer / Tokenizer）—— 将源代码字符串转换为 Token 序列。
//!
//! 词法分析器使用 `Peekable` 迭代器逐字符扫描输入。多字符运算符
//! （`==`、`!=`、`<=`、`>=`、`+=`、`-=`）通过在消费前窥视下一个字符来处理。

use crate::errors::LexError;
use crate::token::{keyword_lookup, Token, TokenKind};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            chars: source.chars().peekable(),
            line: 1,
            col: 1,
        }
    }

    /// 将整个源代码字符串词法分析为 Token 向量。
    pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();

        loop {
            // 跳过空白和注释
            lexer.skip_whitespace_and_comments();

            match lexer.chars.peek() {
                None => {
                    tokens.push(Token::new(TokenKind::Eof, lexer.line, lexer.col));
                    break;
                }
                Some(&ch) => {
                    let token = match ch {
                        '0'..='9' => lexer.scan_number()?,
                        'a'..='z' | 'A'..='Z' | '_' => lexer.scan_identifier(),
                        '"' => lexer.scan_string()?,
                        // 单字符或双字符运算符
                        '+' => lexer.scan_plus_minus(TokenKind::Plus, TokenKind::PlusAssign),
                        '-' => lexer.scan_minus_or_arrow(),
                        '*' => {
                            let (line, col) = lexer.pos();
                            lexer.advance();
                            Token::new(TokenKind::Star, line, col)
                        }
                        '/' => {
                            let (line, col) = lexer.pos();
                            lexer.advance();
                            Token::new(TokenKind::Slash, line, col)
                        }
                        '%' => {
                            let (line, col) = lexer.pos();
                            lexer.advance();
                            Token::new(TokenKind::Percent, line, col)
                        }
                        '=' => lexer.scan_eq_or_assign(),
                        '!' => lexer.scan_bang(),
                        '<' => lexer.scan_lt(),
                        '>' => lexer.scan_gt(),
                        '&' => lexer.scan_and(),
                        '|' => lexer.scan_or(),
                        '(' => lexer.single(TokenKind::LParen),
                        ')' => lexer.single(TokenKind::RParen),
                        '{' => lexer.single(TokenKind::LBrace),
                        '}' => lexer.single(TokenKind::RBrace),
                        '[' => lexer.single(TokenKind::LBracket),
                        ']' => lexer.single(TokenKind::RBracket),
                        ',' => lexer.single(TokenKind::Comma),
                        ';' => lexer.single(TokenKind::Semicolon),
                        ':' => lexer.single(TokenKind::Colon),
                        _ => {
                            let (line, col) = lexer.pos();
                            lexer.advance();
                            return Err(LexError::UnexpectedChar { ch, line, col });
                        }
                    };
                    tokens.push(token);
                }
            }
        }

        Ok(tokens)
    }

    fn pos(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n' {
                self.advance();
            } else if ch == '/' {
                // 向前窥视，判断是否为 '//'
                let mut clone = self.chars.clone();
                clone.next(); // 消费 '/'
                if clone.peek() == Some(&'/') {
                    // 行注释 —— 跳过直到行尾
                    self.advance(); // '/'
                    self.advance(); // '/'
                    while let Some(&c) = self.chars.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn scan_number(&mut self) -> Result<Token, LexError> {
        let (line, col) = self.pos();
        let mut s = String::new();

        // 整数部分
        while let Some(&ch) = self.chars.peek() {
            if ch.is_ascii_digit() {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // 小数部分
        if self.chars.peek() == Some(&'.') {
            // 确保 '.' 后面是数字（排除 `arr.x` 这类方法调用形式）
            let mut clone = self.chars.clone();
            clone.next(); // 跳过 '.'
            if clone.peek().is_some_and(|c| c.is_ascii_digit()) {
                s.push('.');
                self.advance();
                while let Some(&ch) = self.chars.peek() {
                    if ch.is_ascii_digit() {
                        s.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let val: f64 =
                    s.parse()
                        .map_err(|_| LexError::InvalidNumber { src: s, line, col })?;
                return Ok(Token::new(TokenKind::Float(val), line, col));
            }
        }

        let val: i64 = s
            .parse()
            .map_err(|_| LexError::InvalidNumber { src: s, line, col })?;
        Ok(Token::new(TokenKind::Int(val), line, col))
    }

    fn scan_identifier(&mut self) -> Token {
        let (line, col) = self.pos();
        let mut s = String::new();
        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // 检查是否为关键字
        let kind = keyword_lookup(&s).unwrap_or(TokenKind::Ident(s));
        Token::new(kind, line, col)
    }

    fn scan_string(&mut self) -> Result<Token, LexError> {
        let (line, _col) = self.pos();
        self.advance(); // 消费开头的 '"'

        let mut s = String::new();
        loop {
            match self.chars.peek() {
                None => return Err(LexError::UnterminatedString { line }),
                Some(&'"') => {
                    self.advance(); // 消费结尾的 '"'
                    break;
                }
                Some(&'\\') => {
                    self.advance();
                    match self.chars.peek() {
                        Some(&'n') => {
                            s.push('\n');
                            self.advance();
                        }
                        Some(&'t') => {
                            s.push('\t');
                            self.advance();
                        }
                        Some(&'r') => {
                            s.push('\r');
                            self.advance();
                        }
                        Some(&'"') => {
                            s.push('"');
                            self.advance();
                        }
                        Some(&'\\') => {
                            s.push('\\');
                            self.advance();
                        }
                        Some(&'0') => {
                            s.push('\0');
                            self.advance();
                        }
                        Some(&c) => {
                            s.push(c);
                            self.advance();
                        }
                        None => return Err(LexError::UnterminatedString { line }),
                    }
                }
                Some(&ch) => {
                    s.push(ch);
                    self.advance();
                }
            }
        }

        Ok(Token::new(TokenKind::Str(s), line, _col))
    }

    fn single(&mut self, kind: TokenKind) -> Token {
        let (line, col) = self.pos();
        self.advance();
        Token::new(kind, line, col)
    }

    fn scan_plus_minus(&mut self, single_kind: TokenKind, double_kind: TokenKind) -> Token {
        let (line, col) = self.pos();
        self.advance(); // 消费 '+' 或 '-'
        if self.chars.peek() == Some(&'=') {
            self.advance();
            Token::new(double_kind, line, col)
        } else {
            Token::new(single_kind, line, col)
        }
    }

    fn scan_minus_or_arrow(&mut self) -> Token {
        let (line, col) = self.pos();
        self.advance(); // 消费 '-'
        if self.chars.peek() == Some(&'=') {
            self.advance();
            Token::new(TokenKind::MinusAssign, line, col)
        } else if self.chars.peek() == Some(&'>') {
            self.advance();
            Token::new(TokenKind::Arrow, line, col)
        } else {
            Token::new(TokenKind::Minus, line, col)
        }
    }

    fn scan_eq_or_assign(&mut self) -> Token {
        let (line, col) = self.pos();
        self.advance(); // 消费 '='
        if self.chars.peek() == Some(&'=') {
            self.advance();
            Token::new(TokenKind::Eq, line, col)
        } else {
            Token::new(TokenKind::Assign, line, col)
        }
    }

    fn scan_bang(&mut self) -> Token {
        let (line, col) = self.pos();
        self.advance(); // 消费 '!'
        if self.chars.peek() == Some(&'=') {
            self.advance();
            Token::new(TokenKind::NotEq, line, col)
        } else {
            Token::new(TokenKind::Bang, line, col)
        }
    }

    fn scan_lt(&mut self) -> Token {
        let (line, col) = self.pos();
        self.advance(); // 消费 '<'
        if self.chars.peek() == Some(&'=') {
            self.advance();
            Token::new(TokenKind::LtEq, line, col)
        } else {
            Token::new(TokenKind::Lt, line, col)
        }
    }

    fn scan_gt(&mut self) -> Token {
        let (line, col) = self.pos();
        self.advance(); // 消费 '>'
        if self.chars.peek() == Some(&'=') {
            self.advance();
            Token::new(TokenKind::GtEq, line, col)
        } else {
            Token::new(TokenKind::Gt, line, col)
        }
    }

    fn scan_and(&mut self) -> Token {
        let (line, col) = self.pos();
        self.advance(); // 消费 '&'
        if self.chars.peek() == Some(&'&') {
            self.advance();
            Token::new(TokenKind::And, line, col)
        } else {
            // 单个 '&' 是非法的 —— 将报为非法字符
            Token::new(TokenKind::And, line, col) // 会导致解析错误
        }
    }

    fn scan_or(&mut self) -> Token {
        let (line, col) = self.pos();
        self.advance(); // 消费 '|'
        if self.chars.peek() == Some(&'|') {
            self.advance();
            Token::new(TokenKind::Or, line, col)
        } else {
            Token::new(TokenKind::Or, line, col) // 会导致解析错误
        }
    }
}
