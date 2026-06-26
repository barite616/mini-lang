//! 解释器 —— 执行 AST 的树遍历求值引擎。
//!
//! 解释器有两个入口：
//!   - `eval(expr, env)` → 求值表达式，返回 `Value`
//!   - `exec(stmt, env)`  → 执行语句，返回 `Flow`
//!
//! `Flow` 将控制流信号（`return`、`break`、`continue`）编码为数据，
//! 这是 Rust 中替代异常机制的习惯用法。

use crate::ast::*;
use crate::environment::Env;
use crate::errors::RuntimeError;
use crate::value::Value;
use std::cell::RefCell;
use std::rc::Rc;

/// 语句执行返回的控制流信号。
#[derive(Debug, Clone)]
pub enum Flow {
    /// 正常完成 —— 继续执行下一条语句。
    Normal,
    /// `return expr;` —— 向上传播到外层函数调用。
    Return(Value),
    /// `break;` —— 向上传播到外层循环。
    Break,
    /// `continue;` —— 向上传播到外层循环。
    Continue,
}

pub struct Interpreter {
    /// 全局环境，所有 REPL 输入共享。
    pub global: Env,
}

impl Interpreter {
    /// 创建新的解释器，全局环境预装内置原生函数。
    pub fn new() -> Self {
        let global = crate::environment::Environment::new().into_rc();
        let interp = Interpreter { global };
        interp.register_builtins();
        interp
    }

    fn register_builtins(&self) {
        use Value::NativeFunc;
        self.global.define(
            "print".to_string(),
            NativeFunc {
                name: "print".to_string(),
                arity: Some(1),
                func: builtin_print,
            },
        );
        self.global.define(
            "println".to_string(),
            NativeFunc {
                name: "println".to_string(),
                arity: Some(1),
                func: builtin_println,
            },
        );
        self.global.define(
            "len".to_string(),
            NativeFunc {
                name: "len".to_string(),
                arity: Some(1),
                func: builtin_len,
            },
        );
        self.global.define(
            "push".to_string(),
            NativeFunc {
                name: "push".to_string(),
                arity: Some(2),
                func: builtin_push,
            },
        );
        self.global.define(
            "pop".to_string(),
            NativeFunc {
                name: "pop".to_string(),
                arity: Some(1),
                func: builtin_pop,
            },
        );
        self.global.define(
            "int".to_string(),
            NativeFunc {
                name: "int".to_string(),
                arity: Some(1),
                func: builtin_int,
            },
        );
        self.global.define(
            "str".to_string(),
            NativeFunc {
                name: "str".to_string(),
                arity: Some(1),
                func: builtin_str,
            },
        );
        self.global.define(
            "type".to_string(),
            NativeFunc {
                name: "type".to_string(),
                arity: Some(1),
                func: builtin_type,
            },
        );
    }

    /// 在全局环境中执行完整程序（语句列表）。
    pub fn run_program(&self, program: &Program) -> Result<Flow, RuntimeError> {
        self.exec_block(&program.statements, &self.global)
    }

    /// 在共享环境中执行语句列表。
    /// 向上传播 `return`/`break`/`continue` 信号。
    pub fn exec_block(&self, stmts: &[Stmt], env: &Env) -> Result<Flow, RuntimeError> {
        for stmt in stmts {
            match self.exec(stmt, env)? {
                Flow::Normal => continue,
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    // ─── 语句执行 ──────────────────────────────────────────────

    pub fn exec(&self, stmt: &Stmt, env: &Env) -> Result<Flow, RuntimeError> {
        match stmt {
            Stmt::Let { name, value } => {
                let v = self.eval(value, env)?;
                env.define(name.clone(), v);
                Ok(Flow::Normal)
            }
            Stmt::Const { name, value } => {
                let v = self.eval(value, env)?;
                env.define_const(name.clone(), v);
                Ok(Flow::Normal)
            }
            Stmt::Assign { target, value } => {
                let v = self.eval(value, env)?;
                self.assign_target(target, v, env)?;
                Ok(Flow::Normal)
            }
            Stmt::ExprStmt(expr) => {
                self.eval(expr, env)?;
                Ok(Flow::Normal)
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let c = self.eval(cond, env)?;
                if c.is_truthy() {
                    self.exec_block(then_branch, env)
                } else if let Some(else_b) = else_branch {
                    self.exec_block(else_b, env)
                } else {
                    Ok(Flow::Normal)
                }
            }
            Stmt::While { cond, body } => {
                loop {
                    let c = self.eval(cond, env)?;
                    if !c.is_truthy() {
                        break;
                    }
                    match self.exec_block(body, env)? {
                        Flow::Normal | Flow::Continue => continue,
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::For {
                var,
                iterable,
                body,
            } => {
                let iter_val = self.eval(iterable, env)?;
                let items = match &iter_val {
                    Value::Array(arr) => arr.borrow().clone(),
                    Value::Str(s) => s.chars().map(|c| Value::Str(c.to_string())).collect(),
                    _ => return Err(RuntimeError::NotIterable),
                };

                for item in items {
                    let loop_env = env.child();
                    loop_env.define(var.clone(), item);
                    match self.exec_block(body, &loop_env)? {
                        Flow::Normal | Flow::Continue => continue,
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::Fn { name, params, body } => {
                let func = Value::Func {
                    params: params.clone(),
                    body: body.clone(),
                    closure: Rc::clone(env),
                };
                env.define(name.clone(), func);
                Ok(Flow::Normal)
            }
            Stmt::Return(expr) => {
                let v = match expr {
                    Some(e) => self.eval(e, env)?,
                    None => Value::Null,
                };
                Ok(Flow::Return(v))
            }
            Stmt::Break => Ok(Flow::Break),
            Stmt::Continue => Ok(Flow::Continue),
        }
    }

    /// 处理对不同目标的赋值：简单变量、数组索引。
    fn assign_target(&self, target: &Expr, value: Value, env: &Env) -> Result<(), RuntimeError> {
        match target {
            Expr::Ident(name) => {
                env.assign(name, value)?;
            }
            Expr::Index { target, index } => {
                let target_val = self.eval(target, env)?;
                let index_val = self.eval(index, env)?;
                match (&target_val, &index_val) {
                    (Value::Array(arr), Value::Int(i)) => {
                        let idx = *i as usize;
                        let mut borrowed = arr.borrow_mut();
                        if idx >= borrowed.len() {
                            return Err(RuntimeError::IndexError {
                                msg: format!(
                                    "index {} out of bounds for array of length {}",
                                    idx,
                                    borrowed.len()
                                ),
                            });
                        }
                        borrowed[idx] = value;
                    }
                    _ => {
                        return Err(RuntimeError::TypeError {
                            msg: format!(
                                "cannot index-assign on {} with {}",
                                target_val.type_name(),
                                index_val.type_name()
                            ),
                        })
                    }
                }
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "invalid assignment target".to_string(),
                })
            }
        }
        Ok(())
    }

    // ─── 表达式求值 ────────────────────────────────────────────

    pub fn eval(&self, expr: &Expr, env: &Env) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Null => Ok(Value::Null),
            Expr::Ident(name) => env
                .get(name)
                .ok_or_else(|| RuntimeError::UndefinedVar(name.clone())),
            Expr::Array(elements) => {
                let mut vals = Vec::with_capacity(elements.len());
                for e in elements {
                    vals.push(self.eval(e, env)?);
                }
                Ok(Value::Array(Rc::new(RefCell::new(vals))))
            }
            Expr::Index { target, index } => {
                let target_val = self.eval(target, env)?;
                let index_val = self.eval(index, env)?;
                self.eval_index(target_val, index_val)
            }
            Expr::BinOp { op, left, right } => {
                // && 和 || 的短路求值
                match op {
                    BinOp::And => {
                        let l = self.eval(left, env)?;
                        if !l.is_truthy() {
                            return Ok(Value::Bool(false));
                        }
                        let r = self.eval(right, env)?;
                        Ok(Value::Bool(r.is_truthy()))
                    }
                    BinOp::Or => {
                        let l = self.eval(left, env)?;
                        if l.is_truthy() {
                            return Ok(Value::Bool(true));
                        }
                        let r = self.eval(right, env)?;
                        Ok(Value::Bool(r.is_truthy()))
                    }
                    _ => {
                        let l = self.eval(left, env)?;
                        let r = self.eval(right, env)?;
                        self.eval_binop(op, l, r)
                    }
                }
            }
            Expr::UnaryOp { op, operand } => {
                let v = self.eval(operand, env)?;
                match op {
                    UnaryOp::Neg => match v {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(RuntimeError::TypeError {
                            msg: format!("cannot negate {}", v.type_name()),
                        }),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!v.is_truthy())),
                }
            }
            Expr::Call { callee, args } => {
                // 求值被调用者
                let func = self.eval(callee, env)?;

                // 求值所有参数
                let mut arg_vals = Vec::with_capacity(args.len());
                for arg in args {
                    arg_vals.push(self.eval(arg, env)?);
                }

                self.call_function(func, arg_vals)
            }
        }
    }

    fn eval_index(&self, target: Value, index: Value) -> Result<Value, RuntimeError> {
        match (&target, &index) {
            (Value::Array(arr), Value::Int(i)) => {
                let borrowed = arr.borrow();
                if *i < 0 {
                    // 负索引：从末尾开始计数
                    let real_idx = borrowed.len() as i64 + *i;
                    if real_idx < 0 {
                        return Err(RuntimeError::IndexError {
                            msg: format!("index {} out of bounds", i),
                        });
                    }
                    return Ok(borrowed[real_idx as usize].clone());
                }
                let idx = *i as usize;
                if idx >= borrowed.len() {
                    return Err(RuntimeError::IndexError {
                        msg: format!(
                            "index {} out of bounds for array of length {}",
                            idx,
                            borrowed.len()
                        ),
                    });
                }
                Ok(borrowed[idx].clone())
            }
            (Value::Str(s), Value::Int(i)) => {
                let chars: Vec<char> = s.chars().collect();
                let idx = if *i < 0 { chars.len() as i64 + *i } else { *i };
                if idx < 0 || idx as usize >= chars.len() {
                    return Err(RuntimeError::IndexError {
                        msg: format!(
                            "index {} out of bounds for string of length {}",
                            i,
                            chars.len()
                        ),
                    });
                }
                Ok(Value::Str(chars[idx as usize].to_string()))
            }
            _ => Err(RuntimeError::TypeError {
                msg: format!(
                    "cannot index {} with {}",
                    target.type_name(),
                    index.type_name()
                ),
            }),
        }
    }

    fn eval_binop(&self, op: &BinOp, l: Value, r: Value) -> Result<Value, RuntimeError> {
        match op {
            BinOp::Add => self.binop_add(l, r),
            BinOp::Sub => self.binop_arith(l, r, |a, b| a - b, |a, b| a - b, "subtract"),
            BinOp::Mul => self.binop_arith(l, r, |a, b| a * b, |a, b| a * b, "multiply"),
            BinOp::Div => self.binop_div(l, r),
            BinOp::Mod => self.binop_mod(l, r),
            BinOp::Eq => Ok(Value::Bool(l == r)),
            BinOp::NotEq => Ok(Value::Bool(l != r)),
            BinOp::Lt => self.binop_compare(l, r, |a, b| a < b, |a, b| a < b, "compare"),
            BinOp::Gt => self.binop_compare(l, r, |a, b| a > b, |a, b| a > b, "compare"),
            BinOp::LtEq => self.binop_compare(l, r, |a, b| a <= b, |a, b| a <= b, "compare"),
            BinOp::GtEq => self.binop_compare(l, r, |a, b| a >= b, |a, b| a >= b, "compare"),
            BinOp::And | BinOp::Or => unreachable!("&& and || are handled with short-circuit"),
        }
    }

    fn binop_add(&self, l: Value, r: Value) -> Result<Value, RuntimeError> {
        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
            (Value::Str(a), other) => Ok(Value::Str(format!("{}{}", a, other.display()))),
            (other, Value::Str(b)) => Ok(Value::Str(format!("{}{}", other.display(), b))),
            _ => Err(RuntimeError::TypeError {
                msg: format!("cannot add {} and {}", l.type_name(), r.type_name()),
            }),
        }
    }

    fn binop_arith(
        &self,
        l: Value,
        r: Value,
        int_fn: fn(i64, i64) -> i64,
        float_fn: fn(f64, f64) -> f64,
        op_name: &str,
    ) -> Result<Value, RuntimeError> {
        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(int_fn(*a, *b))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_fn(*a, *b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(float_fn(*a as f64, *b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(float_fn(*a, *b as f64))),
            _ => Err(RuntimeError::TypeError {
                msg: format!("cannot {} {} and {}", op_name, l.type_name(), r.type_name()),
            }),
        }
    }

    fn binop_div(&self, l: Value, r: Value) -> Result<Value, RuntimeError> {
        match (&l, &r) {
            (Value::Int(_), Value::Int(0)) => Err(RuntimeError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            }
            _ => Err(RuntimeError::TypeError {
                msg: format!("cannot divide {} by {}", l.type_name(), r.type_name()),
            }),
        }
    }

    fn binop_mod(&self, l: Value, r: Value) -> Result<Value, RuntimeError> {
        match (&l, &r) {
            (Value::Int(_), Value::Int(0)) => Err(RuntimeError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 % b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a % *b as f64)),
            _ => Err(RuntimeError::TypeError {
                msg: format!("cannot mod {} by {}", l.type_name(), r.type_name()),
            }),
        }
    }

    fn binop_compare(
        &self,
        l: Value,
        r: Value,
        int_fn: fn(i64, i64) -> bool,
        float_fn: fn(f64, f64) -> bool,
        _op_name: &str,
    ) -> Result<Value, RuntimeError> {
        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(int_fn(*a, *b))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(float_fn(*a, *b))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(float_fn(*a as f64, *b))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(float_fn(*a, *b as f64))),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a < b)), // 字典序比较
            _ => Err(RuntimeError::TypeError {
                msg: format!("cannot compare {} and {}", l.type_name(), r.type_name()),
            }),
        }
    }

    /// 调用函数值（用户定义或原生函数），参数已求值完毕。
    fn call_function(&self, func: Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match func {
            Value::Func {
                params,
                body,
                closure,
            } => {
                if params.len() != args.len() {
                    return Err(RuntimeError::ArgCountMismatch {
                        expected: params.len(),
                        got: args.len(),
                    });
                }
                // 创建新作用域，作为函数闭包的子作用域。
                // 这正是闭包生效的关键：函数记住的是定义时的环境，
                // 而非调用时的环境。
                let call_env = closure.child();
                for (param, arg) in params.iter().zip(args.into_iter()) {
                    call_env.define(param.clone(), arg);
                }
                match self.exec_block(&body, &call_env)? {
                    Flow::Return(v) => Ok(v),
                    Flow::Normal => Ok(Value::Null),
                    Flow::Break => Err(RuntimeError::InvalidBreak),
                    Flow::Continue => Err(RuntimeError::InvalidContinue),
                }
            }
            Value::NativeFunc { arity, func, .. } => {
                if let Some(expected) = arity {
                    if args.len() != expected {
                        return Err(RuntimeError::ArgCountMismatch {
                            expected,
                            got: args.len(),
                        });
                    }
                }
                func(&args)
            }
            _ => Err(RuntimeError::NotCallable),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

// ─── 内置原生函数 ──────────────────────────────────────────────

fn builtin_print(args: &[Value]) -> Result<Value, RuntimeError> {
    print!("{}", args[0].display());
    Ok(Value::Null)
}

fn builtin_println(args: &[Value]) -> Result<Value, RuntimeError> {
    println!("{}", args[0].display());
    Ok(Value::Null)
}

fn builtin_len(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
        Value::Array(arr) => Ok(Value::Int(arr.borrow().len() as i64)),
        other => Err(RuntimeError::TypeError {
            msg: format!("len() does not support {}", other.type_name()),
        }),
    }
}

fn builtin_push(args: &[Value]) -> Result<Value, RuntimeError> {
    match (&args[0], &args[1]) {
        (Value::Array(arr), val) => {
            arr.borrow_mut().push(val.clone());
            Ok(Value::Null)
        }
        _ => Err(RuntimeError::TypeError {
            msg: "push() expects an array and a value".to_string(),
        }),
    }
}

fn builtin_pop(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Array(arr) => arr
            .borrow_mut()
            .pop()
            .ok_or_else(|| RuntimeError::IndexError {
                msg: "pop() on empty array".to_string(),
            }),
        _ => Err(RuntimeError::TypeError {
            msg: "pop() expects an array".to_string(),
        }),
    }
}

fn builtin_int(args: &[Value]) -> Result<Value, RuntimeError> {
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::Str(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| RuntimeError::TypeError {
                msg: format!("cannot convert '{}' to int", s),
            }),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        other => Err(RuntimeError::TypeError {
            msg: format!("cannot convert {} to int", other.type_name()),
        }),
    }
}

fn builtin_str(args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::Str(args[0].display()))
}

fn builtin_type(args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::Str(args[0].type_name().to_string()))
}
