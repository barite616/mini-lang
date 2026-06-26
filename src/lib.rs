//! Mini-Lang —— 自定义语言的小型树遍历解释器。
//!
//! 库根模块，重新导出所有公开模块。

pub mod ast;
pub mod environment;
pub mod errors;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod value;

use errors::MiniError;

/// 运行源代码字符串，走完整流水线：词法分析 → 语法分析 → 解释执行。
/// 使用给定解释器的全局环境（REPL 状态可持久化）。
pub fn run_source(source: &str, interp: &interpreter::Interpreter) -> Result<(), MiniError> {
    let tokens = lexer::Lexer::tokenize(source)?;
    let program = parser::Parser::parse(tokens)?;
    interp.run_program(&program).map_err(MiniError::from)?;
    Ok(())
}

/// 运行源代码并返回最后一个表达式的值（供 REPL 显示）。
pub fn eval_source(
    source: &str,
    interp: &interpreter::Interpreter,
) -> Result<Option<value::Value>, MiniError> {
    let tokens = lexer::Lexer::tokenize(source)?;
    let program = parser::Parser::parse(tokens)?;

    let mut last_value = None;
    for stmt in &program.statements {
        match stmt {
            ast::Stmt::ExprStmt(expr) => {
                let v = interp.eval(expr, &interp.global).map_err(MiniError::from)?;
                last_value = Some(v);
            }
            other => {
                interp
                    .run_program(&ast::Program {
                        statements: vec![other.clone()],
                    })
                    .map_err(MiniError::from)?;
                last_value = None;
            }
        }
    }
    Ok(last_value)
}
