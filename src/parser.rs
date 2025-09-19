use anyhow::{anyhow, Result};
use std::mem::discriminant;

use crate::ast::*;
use crate::token::TokenKind as T;

pub struct Parser {
    toks: Vec<T>,
    i: usize,
}

impl Parser {
    pub fn new(toks: Vec<T>) -> Self { Self { toks, i: 0 } }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut items = Vec::new();
        while !self.is(T::Eof) {
            if self.is(T::KwClass) {
                items.push(Item::Class(self.parse_class()?));
            } else if self.is(T::KwFn) {
                items.push(Item::Func(self.parse_func()?));
            } else if self.is(T::KwLet) {
                let decl = self.parse_let_decl()?;
                self.expect(T::Semicolon)?;
                items.push(Item::Let(decl));
            } else {
                let stmt = self.parse_stmt()?;
                match stmt {
                    Stmt::Let(d) => items.push(Item::Let(d)),
                    _ => return Err(anyhow!("Csak class/fn/let engedett a toplevelen")),
                }
            }
        }
        Ok(Program { items })
    }

    fn parse_class(&mut self) -> Result<ClassDecl> {
        self.expect(T::KwClass)?;
        let name = self.expect_ident()?;
        self.expect(T::LBrace)?;
        let mut methods = Vec::new();
        while !self.is(T::RBrace) {
            self.expect(T::KwFn)?;
            methods.push(self.parse_func_after_kwfn()?);
        }
        self.expect(T::RBrace)?;
        Ok(ClassDecl { name, methods })
    }

    fn parse_func(&mut self) -> Result<FuncDecl> {
        self.expect(T::KwFn)?;
        self.parse_func_after_kwfn()
    }

    fn parse_func_after_kwfn(&mut self) -> Result<FuncDecl> {
        let name = self.expect_ident()?;
        self.expect(T::LParen)?;
        let mut params = Vec::new();
        if !self.is(T::RParen) {
            loop {
                params.push(self.expect_ident()?);
                if self.eat(T::Comma) { continue; }
                break;
            }
        }
        self.expect(T::RParen)?;
        let body = self.parse_block()?;
        Ok(FuncDecl { name, params, body })
    }

    fn parse_block(&mut self) -> Result<Block> {
        self.expect(T::LBrace)?;
        let mut stmts = Vec::new();
        while !self.is(T::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(T::RBrace)?;
        Ok(Block { stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        if self.is(T::KwLet) {
            let d = self.parse_let_decl()?;
            self.expect(T::Semicolon)?;
            return Ok(Stmt::Let(d));
        }
        if self.is(T::KwReturn) {
            self.bump();
            if self.is(T::Semicolon) { self.bump(); return Ok(Stmt::Return(None)); }
            let e = self.parse_expr()?;
            self.expect(T::Semicolon)?;
            return Ok(Stmt::Return(Some(e)));
        }
        if self.is(T::KwIf) {
            self.bump();
            self.expect(T::LParen)?;
            let cond = self.parse_expr()?;
            self.expect(T::RParen)?;
            let then_block = self.parse_block()?;
            let else_block = if self.eat(T::KwElse) { Some(self.parse_block()?) } else { None };
            return Ok(Stmt::If { cond, then_block, else_block });
        }
        if self.is(T::KwWhile) {
            self.bump();
            self.expect(T::LParen)?;
            let cond = self.parse_expr()?;
            self.expect(T::RParen)?;
            let body = self.parse_block()?;
            return Ok(Stmt::While { cond, body });
        }
        if matches!(self.peek(), T::Ident(_)) && self.kind_eq(self.peek_n(1), &T::Assign) {
            let name = if let T::Ident(s) = self.peek().clone() { s } else { unreachable!() };
            self.bump();
            self.expect(T::Assign)?;
            let value = self.parse_expr()?;
            self.expect(T::Semicolon)?;
            return Ok(Stmt::Assign { name, value });
        }
        let e = self.parse_expr()?;
        self.expect(T::Semicolon)?;
        Ok(Stmt::Expr(e))
    }

    fn parse_let_decl(&mut self) -> Result<LetDecl> {
        self.expect(T::KwLet)?;
        let name = self.expect_ident()?;
        self.expect(T::Assign)?;
        let init = self.parse_expr()?;
        Ok(LetDecl { name, init })
    }

    // ---- Expressions ----
    fn parse_expr(&mut self) -> Result<Expr> { self.parse_equality() }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;
        loop {
            if self.eat(T::Eq) {
                let right = self.parse_comparison()?;
                left = Expr::Binary { op: BinOp::Eq, left: Box::new(left), right: Box::new(right) };
            } else if self.eat(T::Ne) {
                let right = self.parse_comparison()?;
                left = Expr::Binary { op: BinOp::Ne, left: Box::new(left), right: Box::new(right) };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_term()?;
        loop {
            if self.eat(T::Lt) {
                let right = self.parse_term()?;
                left = Expr::Binary { op: BinOp::Lt, left: Box::new(left), right: Box::new(right) };
            } else if self.eat(T::Le) {
                let right = self.parse_term()?;
                left = Expr::Binary { op: BinOp::Le, left: Box::new(left), right: Box::new(right) };
            } else if self.eat(T::Gt) {
                let right = self.parse_term()?;
                left = Expr::Binary { op: BinOp::Gt, left: Box::new(left), right: Box::new(right) };
            } else if self.eat(T::Ge) {
                let right = self.parse_term()?;
                left = Expr::Binary { op: BinOp::Ge, left: Box::new(left), right: Box::new(right) };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut left = self.parse_factor()?;
        loop {
            if self.eat(T::Plus) {
                let right = self.parse_factor()?;
                left = Expr::Binary { op: BinOp::Add, left: Box::new(left), right: Box::new(right) };
            } else if self.eat(T::Minus) {
                let right = self.parse_factor()?;
                left = Expr::Binary { op: BinOp::Sub, left: Box::new(left), right: Box::new(right) };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut left = self.parse_postfix()?;
        loop {
            if self.eat(T::Star) {
                let right = self.parse_postfix()?;
                left = Expr::Binary { op: BinOp::Mul, left: Box::new(left), right: Box::new(right) };
            } else if self.eat(T::Slash) {
                let right = self.parse_postfix()?;
                left = Expr::Binary { op: BinOp::Div, left: Box::new(left), right: Box::new(right) };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.eat(T::LParen) {
                let mut args = Vec::new();
                if !self.is(T::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if self.eat(T::Comma) { continue; }
                        break;
                    }
                }
                self.expect(T::RParen)?;
                expr = Expr::Call { callee: Box::new(expr), args };
            } else if self.eat(T::LBracket) {
                let idx = self.parse_expr()?;
                self.expect(T::RBracket)?;
                expr = Expr::Index { target: Box::new(expr), index: Box::new(idx) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.peek().clone() {
            T::Ident(name) => { self.bump(); Ok(Expr::Ident(name)) }
            T::Int(n) => { self.bump(); Ok(Expr::Int(n)) }
            T::Str(s) => { self.bump(); Ok(Expr::Str(s)) }
            T::KwTrue => { self.bump(); Ok(Expr::Bool(true)) }
            T::KwFalse => { self.bump(); Ok(Expr::Bool(false)) }
            T::LParen => {
                self.bump();
                let e = self.parse_expr()?;
                self.expect(T::RParen)?;
                Ok(Expr::Group(Box::new(e)))
            }
            T::LBracket => {
                self.bump();
                let mut elems = Vec::new();
                if !self.is(T::RBracket) {
                    loop {
                        elems.push(self.parse_expr()?);
                        if self.eat(T::Comma) { continue; }
                        break;
                    }
                }
                self.expect(T::RBracket)?;
                Ok(Expr::Array(elems))
            }
            other => Err(anyhow!(format!("Várt elsődleges kifejezés, kaptam: {:?}", other))),
        }
    }

    // ---- helpers ----
    fn peek(&self) -> &T { self.toks.get(self.i).unwrap_or(&T::Eof) }
    fn peek_n(&self, n: usize) -> &T { self.toks.get(self.i + n).unwrap_or(&T::Eof) }

    fn is(&self, k: T) -> bool { discriminant(self.peek()) == discriminant(&k) }
    fn kind_eq(&self, a: &T, b: &T) -> bool { discriminant(a) == discriminant(b) }

    fn eat(&mut self, k: T) -> bool { if self.is(k) { self.i += 1; true } else { false } }

    fn expect(&mut self, k: T) -> Result<()> {
        if self.eat(k.clone()) { Ok(()) } else { Err(anyhow!(format!("Várt token: {:?}, kaptam: {:?}", k, self.peek()))) }
    }

    fn bump(&mut self) { self.i += 1; }

    fn expect_ident(&mut self) -> Result<String> {
        match self.peek().clone() {
            T::Ident(s) => { self.bump(); Ok(s) }
            other => Err(anyhow!(format!("Várt azonosító, kaptam: {:?}", other))),
        }
    }
}
