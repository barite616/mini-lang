//! 语法分析器 —— 递归下降解析器，将 Token 流转换为 AST。
//!
//! 解析器在 Token 向量上维护一个位置游标 `pos`。每条语法规则
//! 实现为一个方法。运算符优先级通过调用层次编码：低优先级运算符
//! 调用高优先级解析器。
//!
//! 优先级（从低到高）：
//!   1. 赋值 (=, +=, -=)         —— 右结合
//!   2. 逻辑或 (||)
//!   3. 逻辑与 (&&)
//!   4. 相等 (==, !=)
//!   5. 比较 (<, >, <=, >=)
//!   6. 项 (+, -)
//!   7. 因子 (*, /, %)
//!   8. 一元 (!, -)
//!   9. 调用 / 索引（后缀）
//!  10. 基础（字面量、标识符、分组）

use crate::ast::*;
use crate::errors::ParseError;
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// 解析一个完整的程序（语句序列）。
    pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
        let mut parser = Parser::new(tokens);
        let mut statements = Vec::new();

        while !parser.is_at_end() {
            statements.push(parser.parse_stmt()?);
        }

        Ok(Program { statements })
    }

    // ─── 游标辅助方法 ─────────────────────────────────────────

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        if !self.is_at_end() {
            self.pos += 1;
        }
        tok
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    /// 若当前 Token 匹配则消费它并返回 true。
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// 若当前 Token 匹配则消费它，否则返回解析错误。
    fn expect(&mut self, kind: &TokenKind, what: &str) -> Result<Token, ParseError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            let tok = self.peek();
            Err(ParseError {
                msg: format!("expected {}, got '{}'", what, tok.kind),
                line: tok.line,
                col: tok.col,
            })
        }
    }

    #[allow(dead_code)]
    fn err(&self, msg: String) -> ParseError {
        let tok = self.peek();
        ParseError {
            msg,
            line: tok.line,
            col: tok.col,
        }
    }

    // ─── 语句解析 ─────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match &self.peek().kind {
            TokenKind::Let => self.parse_let(),
            TokenKind::Const => self.parse_const(),
            TokenKind::Fn => self.parse_fn(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Break => {
                self.advance();
                self.expect(&TokenKind::Semicolon, "';'")?;
                Ok(Stmt::Break)
            }
            TokenKind::Continue => {
                self.advance();
                self.expect(&TokenKind::Semicolon, "';'")?;
                Ok(Stmt::Continue)
            }
            _ => self.parse_expr_or_assign_stmt(),
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // 'let'
        let name = self.parse_ident()?;
        self.expect(&TokenKind::Assign, "'='")?;
        let value = self.parse_expr()?;
        self.expect(&TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Let { name, value })
    }

    fn parse_const(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // 'const'
        let name = self.parse_ident()?;
        self.expect(&TokenKind::Assign, "'='")?;
        let value = self.parse_expr()?;
        self.expect(&TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Const { name, value })
    }

    fn parse_fn(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // 'fn'
        let name = self.parse_ident()?;
        self.expect(&TokenKind::LParen, "'('")?;

        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            params.push(self.parse_ident()?);
            while self.match_token(&TokenKind::Comma) {
                params.push(self.parse_ident()?);
            }
        }
        self.expect(&TokenKind::RParen, "')'")?;
        self.expect(&TokenKind::LBrace, "'{'")?;
        let body = self.parse_block()?;
        Ok(Stmt::Fn { name, params, body })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // 'if'
        self.expect(&TokenKind::LParen, "'('")?;
        let cond = self.parse_expr()?;
        self.expect(&TokenKind::RParen, "')'")?;
        self.expect(&TokenKind::LBrace, "'{'")?;
        let then_branch = self.parse_block()?;

        let else_branch = if self.match_token(&TokenKind::Else) {
            if self.check(&TokenKind::If) {
                // else if —— 包装成只含单个 If 语句的向量
                let nested = self.parse_if()?;
                Some(vec![nested])
            } else {
                self.expect(&TokenKind::LBrace, "'{'")?;
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Stmt::If {
            cond,
            then_branch,
            else_branch,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // 'while'
        self.expect(&TokenKind::LParen, "'('")?;
        let cond = self.parse_expr()?;
        self.expect(&TokenKind::RParen, "')'")?;
        self.expect(&TokenKind::LBrace, "'{'")?;
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // 'for'
        self.expect(&TokenKind::LParen, "'('")?;
        let var = self.parse_ident()?;
        self.expect(&TokenKind::In, "'in'")?;
        let iterable = self.parse_expr()?;
        self.expect(&TokenKind::RParen, "')'")?;
        self.expect(&TokenKind::LBrace, "'{'")?;
        let body = self.parse_block()?;
        Ok(Stmt::For {
            var,
            iterable,
            body,
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // 'return'
        if self.check(&TokenKind::Semicolon) {
            self.advance();
            return Ok(Stmt::Return(None));
        }
        let value = self.parse_expr()?;
        self.expect(&TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Return(Some(value)))
    }

    /// 解析表达式语句或赋值语句。
    /// `x = 5;`、`x += 3;`、`arr[0] = 1;`、`print(x);` 都进入这里。
    fn parse_expr_or_assign_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr()?;

        // 检查是否为赋值
        let stmt = match &self.peek().kind {
            TokenKind::Assign => {
                self.advance();
                let value = self.parse_expr()?;
                Stmt::Assign {
                    target: expr,
                    value,
                }
            }
            TokenKind::PlusAssign => {
                self.advance();
                let value = self.parse_expr()?;
                // 将 `x += v` 脱糖为 `x = x + v`
                let target_clone = expr.clone();
                let combined = Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(target_clone),
                    right: Box::new(value),
                };
                Stmt::Assign {
                    target: expr,
                    value: combined,
                }
            }
            TokenKind::MinusAssign => {
                self.advance();
                let value = self.parse_expr()?;
                let target_clone = expr.clone();
                let combined = Expr::BinOp {
                    op: BinOp::Sub,
                    left: Box::new(target_clone),
                    right: Box::new(value),
                };
                Stmt::Assign {
                    target: expr,
                    value: combined,
                }
            }
            _ => Stmt::ExprStmt(expr),
        };

        self.expect(&TokenKind::Semicolon, "';'")?;
        Ok(stmt)
    }

    /// 解析代码块 `{ stmts... }` —— 假设开头的 `{` 已被消费。
    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&TokenKind::RBrace, "'}'")?;
        Ok(stmts)
    }

    fn parse_ident(&mut self) -> Result<String, ParseError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Ident(name) => Ok(name),
            _ => Err(ParseError {
                msg: format!("expected identifier, got '{}'", tok.kind),
                line: tok.line,
                col: tok.col,
            }),
        }
    }

    // ─── 表达式解析（按优先级，从低到高）─────────────────────

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while self.match_token(&TokenKind::Or) {
            let right = self.parse_and()?;
            left = Expr::BinOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_equality()?;
        while self.match_token(&TokenKind::And) {
            let right = self.parse_equality()?;
            left = Expr::BinOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match &self.peek().kind {
                TokenKind::Eq => BinOp::Eq,
                TokenKind::NotEq => BinOp::NotEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_term()?;
        loop {
            let op = match &self.peek().kind {
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::LtEq => BinOp::LtEq,
                TokenKind::GtEq => BinOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_factor()?;
        loop {
            let op = match &self.peek().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match &self.peek().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let op = match &self.peek().kind {
            TokenKind::Bang => Some(UnaryOp::Not),
            TokenKind::Minus => Some(UnaryOp::Neg),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op,
                operand: Box::new(operand),
            });
        }

        self.parse_postfix()
    }

    /// 解析后缀操作：函数调用 `f(...)` 和索引访问 `arr[...]`。
    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            match &self.peek().kind {
                TokenKind::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        args.push(self.parse_expr()?);
                        while self.match_token(&TokenKind::Comma) {
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(&TokenKind::RParen, "')'")?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(&TokenKind::RBracket, "']'")?;
                    expr = Expr::Index {
                        target: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let tok = self.peek().clone();
        let expr = match &tok.kind {
            TokenKind::Int(n) => {
                self.advance();
                Expr::Int(*n)
            }
            TokenKind::Float(f) => {
                self.advance();
                Expr::Float(*f)
            }
            TokenKind::Str(s) => {
                self.advance();
                Expr::Str(s.clone())
            }
            TokenKind::True => {
                self.advance();
                Expr::Bool(true)
            }
            TokenKind::False => {
                self.advance();
                Expr::Bool(false)
            }
            TokenKind::Null => {
                self.advance();
                Expr::Null
            }
            TokenKind::Ident(name) => {
                self.advance();
                Expr::Ident(name.clone())
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen, "')'")?;
                expr
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    elements.push(self.parse_expr()?);
                    while self.match_token(&TokenKind::Comma) {
                        if self.check(&TokenKind::RBracket) {
                            break; // 尾随逗号
                        }
                        elements.push(self.parse_expr()?);
                    }
                }
                self.expect(&TokenKind::RBracket, "']'")?;
                Expr::Array(elements)
            }
            _ => {
                return Err(ParseError {
                    msg: format!("unexpected token '{}'", tok.kind),
                    line: tok.line,
                    col: tok.col,
                })
            }
        };
        Ok(expr)
    }
}
