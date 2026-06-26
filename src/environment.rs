//! 环境 —— 变量作用域链。
//!
//! 每个 `Environment` 持有一个变量绑定 `HashMap` 和一个可选的
//! 父级（外层作用域）。变量查找沿链向上进行。
//!
//! `Rc<Environment>`（别名为 `Env`）允许多个子作用域共享父级。
//! `RefCell` 提供内部可变性，使我们可以通过共享的 `&Environment`
//! 引用来定义/赋值变量。
//!
//! 这是展示 Rust 所有权模型的核心数据结构：通过 `Rc` 实现共享
//! 所有权，通过 `RefCell` 实现内部可变性，通过 `Option<Rc<_>>`
//! 实现链表式的链结构。

use crate::errors::RuntimeError;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// 共享环境引用的类型别名。
pub type Env = Rc<Environment>;

pub struct Environment {
    /// 变量绑定：名称 → 值。
    vars: RefCell<HashMap<String, Value>>,
    /// 声明为 `const` 的名称 —— 不可重新赋值。
    consts: RefCell<HashMap<String, ()>>,
    /// 外层（父级）作用域，若有。
    parent: Option<Env>,
}

impl Environment {
    /// 创建无父级的新根（全局）环境。
    pub fn new() -> Self {
        Environment {
            vars: RefCell::new(HashMap::new()),
            consts: RefCell::new(HashMap::new()),
            parent: None,
        }
    }

    /// 包装进 `Rc` 以获得可共享的 `Env`。
    pub fn into_rc(self) -> Env {
        Rc::new(self)
    }

    /// 创建以 `self` 为父级的子作用域。
    pub fn child(self: &Env) -> Env {
        Rc::new(Environment {
            vars: RefCell::new(HashMap::new()),
            consts: RefCell::new(HashMap::new()),
            parent: Some(Rc::clone(self)),
        })
    }

    /// 在当前作用域定义新变量。
    pub fn define(&self, name: String, value: Value) {
        self.vars.borrow_mut().insert(name, value);
    }

    /// 定义常量变量。后续 `assign` 调用将报错。
    pub fn define_const(&self, name: String, value: Value) {
        self.consts.borrow_mut().insert(name.clone(), ());
        self.vars.borrow_mut().insert(name, value);
    }

    /// 检查某名称在当前作用域或任意祖先中是否被声明为常量。
    fn is_const(&self, name: &str) -> bool {
        if self.consts.borrow().contains_key(name) {
            return true;
        }
        self.parent.as_ref().is_some_and(|p| p.is_const(name))
    }

    /// 查找变量，沿作用域链向上查找。
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.vars.borrow().get(name) {
            return Some(v.clone());
        }
        self.parent.as_ref().and_then(|p| p.get(name))
    }

    /// 给已存在的变量赋值。沿作用域链向上查找。
    /// 若变量未定义或为常量则返回错误。
    pub fn assign(&self, name: &str, value: Value) -> Result<(), RuntimeError> {
        if self.vars.borrow().contains_key(name) {
            if self.is_const(name) {
                return Err(RuntimeError::ReassignConst(name.to_string()));
            }
            self.vars.borrow_mut().insert(name.to_string(), value);
            return Ok(());
        }
        match &self.parent {
            Some(p) => p.assign(name, value),
            None => Err(RuntimeError::UndefinedVar(name.to_string())),
        }
    }

    /// 获取父环境引用（用于闭包捕获）。
    pub fn parent(&self) -> Option<&Env> {
        self.parent.as_ref()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vars: Vec<String> = self.vars.borrow().keys().cloned().collect();
        write!(f, "Env{{{}}}", vars.join(", "))
    }
}
