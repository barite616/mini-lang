//! 抽象语法树（AST）节点定义。
//!
//! AST 是语法分析器的输出、解释器的输入。所有递归子表达式都使用
//! `Box` 装箱，因为 Rust 在编译期需要知道类型大小，而递归枚举
//! 不使用间接引用时大小是无限的。

/// 二元运算符（大致按优先级排列，便于阅读）。
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

/// 一元运算符。
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// 表达式节点 —— 求值时会产生一个值的任何结构。
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// 整数字面量，例如 `42`
    Int(i64),
    /// 浮点数字面量，例如 `3.14`
    Float(f64),
    /// 字符串字面量，例如 `"hello"`
    Str(String),
    /// 布尔字面量：`true` 或 `false`
    Bool(bool),
    /// `null` 字面量
    Null,
    /// 变量引用，例如 `x`
    Ident(String),
    /// 数组字面量，例如 `[1, 2, 3]`
    Array(Vec<Expr>),
    /// 索引访问，例如 `arr[0]`
    Index { target: Box<Expr>, index: Box<Expr> },
    /// 二元运算，例如 `a + b`
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// 一元运算，例如 `-x` 或 `!flag`
    UnaryOp { op: UnaryOp, operand: Box<Expr> },
    /// 函数调用，例如 `add(1, 2)`
    Call { callee: Box<Expr>, args: Vec<Expr> },
}

/// 语句节点 —— 为副作用而执行的任何结构。
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `let name = value;`
    Let { name: String, value: Expr },
    /// `const name = value;`
    Const { name: String, value: Expr },
    /// `name = value;` 或 `name += value;`
    Assign { target: Expr, value: Expr },
    /// 作为语句的裸表达式，例如 `print(x);`
    ExprStmt(Expr),
    /// `if (cond) { ... } else { ... }`
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    /// `while (cond) { ... }`
    While { cond: Expr, body: Vec<Stmt> },
    /// `for (x in iterable) { ... }`
    For {
        var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    /// `fn name(params) { body }`
    Fn {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    /// `return expr;` 或 `return;`
    Return(Option<Expr>),
    /// `break;`
    Break,
    /// `continue;`
    Continue,
}

/// 一个完整的程序就是一组语句序列。
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
