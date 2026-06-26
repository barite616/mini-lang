//! Mini-Lang REPL 入口。
//!
//! 支持两种模式：
//!   1. 交互式 REPL（默认）—— 带行编辑的读取-求值-打印循环。
//!   2. 文件执行 —— `mini-lang run script.mini` 执行脚本文件。

use mini_lang::interpreter::Interpreter;
use mini_lang::{eval_source, run_source};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::fs;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // 文件执行模式：`mini-lang run <file>`
    if args.len() >= 2 && args[1] == "run" {
        if args.len() < 3 {
            eprintln!("Usage: mini-lang run <file.mini>");
            std::process::exit(1);
        }
        let filename = &args[2];
        let source = fs::read_to_string(filename).unwrap_or_else(|e| {
            eprintln!("Error reading file '{}': {}", filename, e);
            std::process::exit(1);
        });
        let interp = Interpreter::new();
        match run_source(&source, &interp) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // REPL 模式
    repl()
}

fn repl() -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    println!("Type 'exit' or Ctrl-D to quit. Type 'help' for help.");
    println!();

    let interp = Interpreter::new();
    let mut buffer = String::new();
    let mut continuing = false;

    loop {
        let prompt = if continuing { ".. " } else { ">> " };
        let readline = rl.readline(prompt);

        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                // 多行输入：若大括号未闭合则继续读取
                if !continuing && (trimmed == "exit" || trimmed == "quit") {
                    break;
                }
                if !continuing && trimmed == "help" {
                    print_help();
                    continue;
                }

                buffer.push_str(&line);
                buffer.push('\n');

                let brace_count = count_unmatched_braces(&buffer);

                if brace_count > 0 {
                    // 需要更多输入 —— 继续缓冲
                    continuing = true;
                    continue;
                }

                // 已有完整代码片段 —— 求值
                continuing = false;
                let source = std::mem::take(&mut buffer);

                match eval_source(&source, &interp) {
                    Ok(Some(val)) => {
                        // 返回 null 的表达式语句不打印
                        if !matches!(val, mini_lang::value::Value::Null) {
                            println!("=> {}", val.display());
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }

                let _ = rl.add_history_entry(source.trim());
            }
            Err(ReadlineError::Interrupted) => {
                if continuing {
                    // 取消多行输入
                    buffer.clear();
                    continuing = false;
                    println!("(cancelled)");
                    continue;
                }
                println!("Ctrl-C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Bye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

/// 统计缓冲区中未闭合的 `{` 大括号数量，用于检测不完整的代码块。
fn count_unmatched_braces(s: &str) -> i32 {
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut in_comment = false;

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        if in_comment {
            if c == '\n' {
                in_comment = false;
            }
            i += 1;
            continue;
        }

        if in_string {
            if c == '\\' {
                i += 2; // 跳过转义字符
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        // 不在字符串或注释中
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            in_comment = true;
            i += 2;
            continue;
        }
        if c == '"' {
            in_string = true;
            i += 1;
            continue;
        }
        if c == '{' {
            depth += 1;
        }
        if c == '}' {
            depth -= 1;
        }
        i += 1;
    }

    depth
}

fn print_help() {
    println!("Mini-Lang — available features:");
    println!("  Variables:   let x = 10;  const PI = 3.14;");
    println!("  Functions:   fn add(a, b) {{ return a + b; }}");
    println!("  Control:     if (x > 0) {{ ... }} else {{ ... }}");
    println!("  Loops:       while (x > 0) {{ ... }}  /  for (i in arr) {{ ... }}");
    println!("  Arrays:      [1, 2, 3]  —  arr[0]  —  push(arr, v)");
    println!("  Strings:     \"hello\"  —  str(42)  —  len(\"abc\")");
    println!("  Builtins:    print(x)  println(x)  len(x)  int(x)  str(x)  type(x)");
    println!("  Break:       break;  continue;");
    println!("  Comments:    // this is a comment");
}
