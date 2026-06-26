# Mini-Lang — 基于 Rust 的小型编程语言解释器

一个用 Rust 实现的树遍历（tree-walking）解释器，支持自定义语言的词法分析、语法分析和求值执行。

## 项目简介

Mini-Lang 是一门轻量级的命令行编程语言，包含完整的解释器工具链：

```
源代码 → [Lexer 词法分析] → Token流 → [Parser 语法分析] → AST → [Interpreter 求值] → 结果
```

## 语言特性

| 特性 | 示例 |
|------|------|
| 变量与常量 | `let x = 10;` / `const PI = 3.14;` |
| 数据类型 | `int` / `float` / `str` / `bool` / `array` / `null` |
| 算术运算 | `+ - * / %` |
| 比较运算 | `== != < > <= >=` |
| 逻辑运算 | `&& \|\| !`（短路求值）|
| 控制流 | `if/else` / `while` / `for (x in arr)` |
| 函数 | `fn add(a, b) { return a + b; }` |
| 递归 | 支持函数递归调用 |
| 闭包 | 函数捕获定义时所在的作用域 |
| 数组 | `[1, 2, 3]` / `arr[0]` / `arr[-1]`（负索引）|
| 字符串 | `"hello"` / 字符串拼接 / 字符遍历 |
| 复合赋值 | `+=` / `-=` |
| 循环控制 | `break` / `continue` |
| 注释 | `// 单行注释` |
| 内置函数 | `print` `println` `len` `push` `pop` `int` `str` `type` |

## 快速开始

### 编译

```bash
cargo build --release
```

### 交互式 REPL

```bash
cargo run --release
```

REPL 示例：

```
Mini-Lang v0.1.0 — a small Rust interpreter
Type 'exit' or Ctrl-D to quit. Type 'help' for help.

>> let x = 10;
>> let y = 20;
>> x + y;
=> 30
>> fn factorial(n) { if (n <= 1) { return 1; } return n * factorial(n - 1); }
>> factorial(5);
=> 120
>> for (i in [1, 2, 3]) { println(i); }
1
2
3
>> exit
```

### 执行脚本文件

```bash
cargo run -- run examples/fibonacci.mini
```

## 项目结构

```
mini-lang/
├── Cargo.toml              # 项目配置
├── README.md               # 项目文档
├── examples/
│   └── fibonacci.mini      # 示例程序
├── src/
│   ├── main.rs             # 入口 + REPL
│   ├── lib.rs              # 库根模块
│   ├── token.rs            # Token 定义
│   ├── lexer.rs            # 词法分析器
│   ├── ast.rs              # AST 节点定义
│   ├── parser.rs           # 递归下降语法分析器
│   ├── interpreter.rs      # 树遍历求值引擎
│   ├── environment.rs      # 变量环境（作用域链）
│   ├── value.rs            # 运行时值类型
│   └── errors.rs           # 错误处理
└── tests/
    └── integration_test.rs # 集成测试（43 个测试用例）
```

## 架构设计

### 1. 词法分析器 (`lexer.rs`)

逐字符扫描源代码，使用 `Peekable<Chars>` 迭代器。支持多字符运算符（`==`, `!=`, `<=`, `>=`, `+=`, `-=`），字符串转义（`\n \t \" \\`），行注释（`//`）。

### 2. 语法分析器 (`parser.rs`)

手写递归下降解析器，运算符优先级通过函数调用层次编码：

```
parse_expr     → parse_or       (||)
parse_or       → parse_and      (&&)
parse_and      → parse_equality (== !=)
parse_equality → parse_comparison (< > <= >=)
parse_comparison → parse_term   (+ -)
parse_term     → parse_factor   (* / %)
parse_factor   → parse_unary    (! -)
parse_unary    → parse_postfix  (call, index)
parse_postfix  → parse_primary  (literals, ident, grouping)
```

### 3. 解释器 (`interpreter.rs`)

树遍历求值。控制流（`return`/`break`/`continue`）用 `Flow` 枚举编码，避免异常机制：

```rust
enum Flow {
    Normal,
    Return(Value),
    Break,
    Continue,
}
```

### 4. 环境 (`environment.rs`)

作用域链使用 `Rc<Environment>` + `RefCell<HashMap>` 实现：

- `Rc`：多个子作用域共享父作用域
- `RefCell`：内部可变性，运行时通过共享引用修改变量表
- `Option<Rc<Environment>>`：父作用域链，递归查找

闭包通过在函数值中存储定义时的环境 `closure: Env` 实现。

## Rust 特性体现

| Rust 特性 | 在项目中的使用 |
|-----------|---------------|
| `enum` + `match` | Token/AST/Value 节点定义，穷尽模式匹配 |
| 所有权 | AST `Box<Expr>` 递归类型 |
| 智能指针 | `Rc<Environment>` 共享作用域 |
| 内部可变性 | `RefCell<HashMap>` 运行时修改变量 |
| 错误处理 | 三层 `Result<T, E>` + `?` 运算符传播 |
| `trait` | `Display`/`Debug`/`From` 实现 |
| 生命周期 | 环境链中的引用关系 |
| 泛型 | `HashMap<String, Value>` 等 |

## 测试

```bash
cargo test
```

共 43 个集成测试，覆盖：

- 算术运算与优先级
- 变量赋值与作用域
- 控制流（if/while/for/break/continue）
- 函数定义、递归、闭包
- 数组操作
- 字符串操作
- 内置函数
- 错误处理（除零、未定义变量、类型错误等）
- 复杂程序（冒泡排序、GCD、闰年判断）

## 工程规范

```bash
cargo fmt      # 代码格式化
cargo clippy   # 静态检查（零警告）
cargo test     # 全部测试通过
```

## 依赖

| 依赖 | 用途 |
|------|------|
| `rustyline` | REPL 行编辑（方向键、历史记录）|
