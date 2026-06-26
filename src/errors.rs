//! 三个流水线阶段的错误类型：词法分析、语法分析、运行时。
//!
//! 每个阶段有自己的错误类型。它们在 `MiniError` 下统一，
//! 使得顶层 `run` 函数可以通过 `?` 运算符传播任意阶段的错误。

use std::fmt;

// ─── 词法错误 ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum LexError {
    UnexpectedChar {
        ch: char,
        line: usize,
        col: usize,
    },
    UnterminatedString {
        line: usize,
    },
    InvalidNumber {
        src: String,
        line: usize,
        col: usize,
    },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedChar { ch, line, col } => {
                write!(
                    f,
                    "Lex error at {}:{}: unexpected character '{}'",
                    line, col, ch
                )
            }
            LexError::UnterminatedString { line } => {
                write!(f, "Lex error at line {}: unterminated string literal", line)
            }
            LexError::InvalidNumber { src, line, col } => {
                write!(f, "Lex error at {}:{}: invalid number '{}'", line, col, src)
            }
        }
    }
}

impl std::error::Error for LexError {}

// ─── 语法错误 ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParseError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at {}:{}: {}", self.line, self.col, self.msg)
    }
}

impl std::error::Error for ParseError {}

// ─── 运行时错误 ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum RuntimeError {
    UndefinedVar(String),
    UndefinedFunc(String),
    TypeError { msg: String },
    DivisionByZero,
    IndexError { msg: String },
    ArgCountMismatch { expected: usize, got: usize },
    NotIndexable,
    NotIterable,
    NotCallable,
    InvalidBreak,
    InvalidContinue,
    ReassignConst(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::UndefinedVar(name) => {
                write!(f, "Runtime error: undefined variable '{}'", name)
            }
            RuntimeError::UndefinedFunc(name) => {
                write!(f, "Runtime error: undefined function '{}'", name)
            }
            RuntimeError::TypeError { msg } => {
                write!(f, "Runtime error: type error: {}", msg)
            }
            RuntimeError::DivisionByZero => {
                write!(f, "Runtime error: division by zero")
            }
            RuntimeError::IndexError { msg } => {
                write!(f, "Runtime error: index error: {}", msg)
            }
            RuntimeError::ArgCountMismatch { expected, got } => {
                write!(
                    f,
                    "Runtime error: expected {} argument(s), got {}",
                    expected, got
                )
            }
            RuntimeError::NotIndexable => {
                write!(f, "Runtime error: this value is not indexable")
            }
            RuntimeError::NotIterable => {
                write!(f, "Runtime error: this value is not iterable")
            }
            RuntimeError::NotCallable => {
                write!(f, "Runtime error: this value is not callable")
            }
            RuntimeError::InvalidBreak => {
                write!(f, "Runtime error: 'break' outside of a loop")
            }
            RuntimeError::InvalidContinue => {
                write!(f, "Runtime error: 'continue' outside of a loop")
            }
            RuntimeError::ReassignConst(name) => {
                write!(f, "Runtime error: cannot reassign const '{}'", name)
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

// ─── 统一错误 ─────────────────────────────────────────────────

#[derive(Debug)]
pub enum MiniError {
    Lex(LexError),
    Parse(ParseError),
    Runtime(RuntimeError),
}

impl fmt::Display for MiniError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MiniError::Lex(e) => write!(f, "{}", e),
            MiniError::Parse(e) => write!(f, "{}", e),
            MiniError::Runtime(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for MiniError {}

impl From<LexError> for MiniError {
    fn from(e: LexError) -> Self {
        MiniError::Lex(e)
    }
}

impl From<ParseError> for MiniError {
    fn from(e: ParseError) -> Self {
        MiniError::Parse(e)
    }
}

impl From<RuntimeError> for MiniError {
    fn from(e: RuntimeError) -> Self {
        MiniError::Runtime(e)
    }
}
