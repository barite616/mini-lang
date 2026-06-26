//! 运行时值类型。
//!
//! `Value` 是表达式求值产生的动态类型。函数值携带闭包（捕获的
//! 环境），从而支持闭包和一等函数。

use crate::ast::Stmt;
use crate::environment::Env;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

/// 运行时值。函数是一等公民 —— 可以存入变量、作为参数传递、
/// 从其他函数返回。
#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Array(Rc<RefCell<Vec<Value>>>),
    Func {
        params: Vec<String>,
        body: Vec<Stmt>,
        closure: Env,
    },
    /// 用 Rust 实现的内置原生函数。
    NativeFunc {
        name: String,
        arity: Option<usize>, // None 表示可变参数
        func: fn(&[Value]) -> Result<Value, crate::errors::RuntimeError>,
    },
    Null,
}

impl Value {
    /// 返回类型名称字符串，用于错误信息。
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Str(_) => "str",
            Value::Bool(_) => "bool",
            Value::Array(_) => "array",
            Value::Func { .. } => "func",
            Value::NativeFunc { .. } => "native_func",
            Value::Null => "null",
        }
    }

    /// 在布尔上下文（if/while 条件）中是否为"真"。
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Array(a) => !a.borrow().is_empty(),
            Value::Null => false,
            Value::Func { .. } | Value::NativeFunc { .. } => true,
        }
    }

    /// 转换为显示字符串（`print` 使用）。
    pub fn display(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => format!("{}", f),
            Value::Str(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr
                    .borrow()
                    .iter()
                    .map(|v| match v {
                        Value::Str(s) => format!("\"{}\"", s),
                        _ => v.display(),
                    })
                    .collect();
                format!("[{}]", items.join(", "))
            }
            Value::Func { params, .. } => format!("<func({})>", params.join(", ")),
            Value::NativeFunc { name, .. } => format!("<native_func {}>", name),
        }
    }

    /// 深拷贝，用于需要完全独立副本的场景。
    /// 数组通过 Rc 共享，因此此方法很少用到。
    pub fn deep_clone(&self) -> Value {
        match self {
            Value::Array(arr) => {
                let cloned: Vec<Value> = arr.borrow().iter().map(|v| v.deep_clone()).collect();
                Value::Array(Rc::new(RefCell::new(cloned)))
            }
            other => other.clone(),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                if a.len() != b.len() {
                    return false;
                }
                a.iter().zip(b.iter()).all(|(x, y)| x == y)
            }
            _ => false,
        }
    }
}
