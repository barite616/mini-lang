//! Integration tests for the Mini-Lang interpreter.
//! Tests cover lexing, parsing, and evaluation end-to-end.

use mini_lang::interpreter::Interpreter;
use mini_lang::value::Value;
use mini_lang::{eval_source, run_source};

fn eval_str(source: &str) -> Result<Option<Value>, mini_lang::errors::MiniError> {
    let interp = Interpreter::new();
    eval_source(source, &interp)
}

fn run_str(source: &str) -> Result<(), mini_lang::errors::MiniError> {
    let interp = Interpreter::new();
    run_source(source, &interp)
}

// ─── Lexer / basic parsing ───────────────────────────────────

#[test]
fn test_basic_arithmetic() {
    let result = eval_str("1 + 2 * 3;").unwrap();
    assert_eq!(result, Some(Value::Int(7)));
}

#[test]
fn test_operator_precedence() {
    // (1 + 2) * 3 should be 9
    assert_eq!(eval_str("(1 + 2) * 3;").unwrap(), Some(Value::Int(9)));
    // 1 + 2 * 3 should be 7 (mult first)
    assert_eq!(eval_str("1 + 2 * 3;").unwrap(), Some(Value::Int(7)));
    // 10 - 2 - 3 should be 5 (left assoc)
    assert_eq!(eval_str("10 - 2 - 3;").unwrap(), Some(Value::Int(5)));
}

#[test]
fn test_unary() {
    assert_eq!(eval_str("-5;").unwrap(), Some(Value::Int(-5)));
    assert_eq!(eval_str("!true;").unwrap(), Some(Value::Bool(false)));
    assert_eq!(eval_str("!!true;").unwrap(), Some(Value::Bool(true)));
    assert_eq!(eval_str("-(-3);").unwrap(), Some(Value::Int(3)));
}

#[test]
fn test_float_arithmetic() {
    assert_eq!(eval_str("2.5;").unwrap(), Some(Value::Float(2.5)));
    assert_eq!(eval_str("1.5 + 2.5;").unwrap(), Some(Value::Float(4.0)));
    assert_eq!(eval_str("10 / 4;").unwrap(), Some(Value::Int(2)));
    assert_eq!(eval_str("10.0 / 4.0;").unwrap(), Some(Value::Float(2.5)));
}

#[test]
fn test_modulo() {
    assert_eq!(eval_str("10 % 3;").unwrap(), Some(Value::Int(1)));
    assert_eq!(eval_str("10 % 2;").unwrap(), Some(Value::Int(0)));
}

// ─── Variables ───────────────────────────────────────────────

#[test]
fn test_variables() {
    let source = r#"
        let x = 10;
        let y = 20;
        x + y;
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(30)));
}

#[test]
fn test_const_cannot_reassign() {
    let source = r#"
        const PI = 3.14;
        PI = 3;
    "#;
    assert!(run_str(source).is_err());
}

#[test]
fn test_assignment() {
    let source = r#"
        let x = 5;
        x = x + 10;
        x;
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(15)));
}

#[test]
fn test_compound_assignment() {
    let source = r#"
        let x = 10;
        x += 5;
        x;
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(15)));
}

// ─── Control flow ────────────────────────────────────────────

#[test]
fn test_if_true() {
    let source = r#"
        let x = 0;
        if (5 > 3) {
            x = 1;
        } else {
            x = 2;
        }
        x;
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(1)));
}

#[test]
fn test_if_else_if() {
    let source = r#"
        let x = 5;
        let result = 0;
        if (x > 10) {
            result = 1;
        } else if (x > 3) {
            result = 2;
        } else {
            result = 3;
        }
        result;
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(2)));
}

#[test]
fn test_while_loop() {
    let source = r#"
        let sum = 0;
        let i = 1;
        while (i <= 10) {
            sum = sum + i;
            i = i + 1;
        }
        sum;
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(55)));
}

#[test]
fn test_while_break() {
    let source = r#"
        let i = 0;
        let result = 0;
        while (true) {
            if (i >= 5) { break; }
            result = result + i;
            i = i + 1;
        }
        result;
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(10)));
}

#[test]
fn test_while_continue() {
    let source = r#"
        let sum = 0;
        let i = 0;
        while (i < 10) {
            i = i + 1;
            if (i % 2 == 0) { continue; }
            sum = sum + i;
        }
        sum;
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(25))); // 1+3+5+7+9
}

// ─── Functions ───────────────────────────────────────────────

#[test]
fn test_function_call() {
    let source = r#"
        fn add(a, b) {
            return a + b;
        }
        add(3, 4);
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(7)));
}

#[test]
fn test_function_no_return() {
    let source = r#"
        fn greet(name) {
            // no return
        }
        greet("world");
    "#;
    let result = eval_str(source).unwrap();
    assert!(matches!(result, Some(Value::Null) | None));
}

#[test]
fn test_nested_functions() {
    let source = r#"
        fn outer(x) {
            fn inner(y) {
                return y * 2;
            }
            return inner(x) + 1;
        }
        outer(10);
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(21)));
}

#[test]
fn test_recursion() {
    let source = r#"
        fn factorial(n) {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5);
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(120)));
}

#[test]
fn test_fibonacci() {
    let source = r#"
        fn fib(n) {
            if (n < 2) {
                return n;
            }
            return fib(n - 1) + fib(n - 2);
        }
        fib(10);
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(55)));
}

#[test]
fn test_closure() {
    let source = r#"
        fn make_counter() {
            let count = 0;
            fn increment() {
                count = count + 1;
                return count;
            }
            return increment;
        }
        let counter = make_counter();
        counter();
        counter();
        counter();
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(3)));
}

// ─── Arrays ──────────────────────────────────────────────────

#[test]
fn test_array_literal() {
    let result = eval_str("[1, 2, 3];").unwrap();
    match result {
        Some(Value::Array(arr)) => {
            let borrowed = arr.borrow();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::Int(1));
            assert_eq!(borrowed[2], Value::Int(3));
        }
        _ => panic!("expected array"),
    }
}

#[test]
fn test_array_index() {
    let source = r#"
        let arr = [10, 20, 30];
        arr[1];
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(20)));
}

#[test]
fn test_array_negative_index() {
    let source = r#"
        let arr = [10, 20, 30];
        arr[-1];
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(30)));
}

#[test]
fn test_array_push() {
    let source = r#"
        let arr = [1, 2];
        push(arr, 3);
        len(arr);
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(3)));
}

#[test]
fn test_array_index_assign() {
    let source = r#"
        let arr = [1, 2, 3];
        arr[0] = 99;
        arr[0];
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(99)));
}

#[test]
fn test_for_array() {
    let source = r#"
        let arr = [1, 2, 3, 4, 5];
        let sum = 0;
        for (x in arr) {
            sum = sum + x;
        }
        sum;
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(15)));
}

// ─── Strings ─────────────────────────────────────────────────

#[test]
fn test_string_concat() {
    assert_eq!(
        eval_str(r#""hello" + " " + "world";"#).unwrap(),
        Some(Value::Str("hello world".to_string()))
    );
}

#[test]
fn test_string_len() {
    assert_eq!(eval_str(r#"len("hello");"#).unwrap(), Some(Value::Int(5)));
}

#[test]
fn test_string_index() {
    assert_eq!(
        eval_str(r#"let s = "abc"; s[0];"#).unwrap(),
        Some(Value::Str("a".to_string()))
    );
}

#[test]
fn test_string_for_loop() {
    let source = r#"
        let s = "abc";
        let count = 0;
        for (c in s) {
            count = count + 1;
        }
        count;
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(3)));
}

// ─── Built-ins & type conversions ────────────────────────────

#[test]
fn test_int_conversion() {
    assert_eq!(eval_str(r#"int("42");"#).unwrap(), Some(Value::Int(42)));
    assert_eq!(eval_str("int(3.9);").unwrap(), Some(Value::Int(3)));
    assert_eq!(eval_str("int(true);").unwrap(), Some(Value::Int(1)));
}

#[test]
fn test_str_conversion() {
    assert_eq!(
        eval_str("str(42);").unwrap(),
        Some(Value::Str("42".to_string()))
    );
}

#[test]
fn test_type_builtin() {
    assert_eq!(
        eval_str("type(42);").unwrap(),
        Some(Value::Str("int".to_string()))
    );
    assert_eq!(
        eval_str(r#"type("hi");"#).unwrap(),
        Some(Value::Str("str".to_string()))
    );
}

// ─── Error cases ─────────────────────────────────────────────

#[test]
fn test_undefined_variable() {
    assert!(eval_str("undefined_var;").is_err());
}

#[test]
fn test_division_by_zero() {
    assert!(eval_str("10 / 0;").is_err());
}

#[test]
fn test_type_error() {
    assert!(eval_str(r#"1 + true;"#).is_err());
    assert!(eval_str(r#"[1,2] - 3;"#).is_err());
}

#[test]
fn test_arg_count_mismatch() {
    let source = r#"
        fn f(a, b) { return a; }
        f(1);
    "#;
    assert!(eval_str(source).is_err());
}

#[test]
fn test_index_out_of_bounds() {
    let source = r#"
        let arr = [1, 2];
        arr[10];
    "#;
    assert!(eval_str(source).is_err());
}

#[test]
fn test_lex_error_unterminated_string() {
    assert!(eval_str(r#""hello"#).is_err());
}

#[test]
fn test_parse_error_missing_semicolon() {
    assert!(eval_str("let x = 5").is_err());
}

// ─── Complex programs ────────────────────────────────────────

#[test]
fn test_greatest_common_divisor() {
    let source = r#"
        fn gcd(a, b) {
            while (b != 0) {
                let temp = b;
                b = a % b;
                a = temp;
            }
            return a;
        }
        gcd(48, 18);
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(6)));
}

#[test]
fn test_bubble_sort() {
    let source = r#"
        fn bubble_sort(arr) {
            let n = len(arr);
            let i = 0;
            while (i < n) {
                let j = 0;
                while (j < n - i - 1) {
                    if (arr[j] > arr[j + 1]) {
                        let temp = arr[j];
                        arr[j] = arr[j + 1];
                        arr[j + 1] = temp;
                    }
                    j = j + 1;
                }
                i = i + 1;
            }
            return arr;
        }
        let sorted = bubble_sort([5, 2, 8, 1, 9, 3]);
        sorted[0];
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Int(1)));
}

#[test]
fn test_leap_year() {
    let source = r#"
        fn is_leap(year) {
            if (year % 400 == 0) {
                return true;
            }
            if (year % 100 == 0) {
                return false;
            }
            if (year % 4 == 0) {
                return true;
            }
            return false;
        }
        is_leap(2000);
    "#;
    assert_eq!(eval_str(source).unwrap(), Some(Value::Bool(true)));

    let source2 = r#"
        fn is_leap(year) {
            if (year % 400 == 0) { return true; }
            if (year % 100 == 0) { return false; }
            if (year % 4 == 0) { return true; }
            return false;
        }
        is_leap(1900);
    "#;
    assert_eq!(eval_str(source2).unwrap(), Some(Value::Bool(false)));
}
