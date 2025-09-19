use std::collections::HashMap;
use anyhow::{anyhow, Result};
use crate::ast::*; use crate::ir::*;

pub struct Codegen { funcs: Vec<FunctionIR> }
impl Codegen {
    pub fn new() -> Self { Self { funcs: Vec::new() } }

    pub fn build(mut self, p: &Program) -> Result<ProgramIR> {
        for it in &p.items {
            match it {
                Item::Func(f) => { self.gen_func(f)?; }
                Item::Class(c) => { for m in &c.methods { let mut m2 = m.clone(); m2.name = format!("{}.{}", c.name, m.name); self.gen_func(&m2)?; } }
                Item::Let(_) => {}
            }
        }
        Ok(ProgramIR { functions: self.funcs })
    }

    fn gen_func(&mut self, f: &FuncDecl) -> Result<()> {
        let mut cg = FnCG::new(&f.params);
        let mut chunk = Chunk::new();
        cg.block(&f.body, &mut chunk)?;
        chunk.code.push(Op::PushVoid);
        chunk.code.push(Op::Return);
        let func = FunctionIR { name: f.name.clone(), arity: f.params.len(), local_count: cg.local_count(), chunk };
        self.funcs.push(func);
        Ok(())
    }
}

struct LoopCtx { start: usize, breaks: Vec<usize>, continues: Vec<usize> }

struct FnCG<'a> {
    locals: HashMap<String, usize>,
    next_local: usize,
    _params: &'a [String],
    loops: Vec<LoopCtx>,
}
impl<'a> FnCG<'a> {
    fn new(params: &'a [String]) -> Self {
        let mut cg = Self { locals: HashMap::new(), next_local: 0, _params: params, loops: Vec::new() };
        for (i, name) in params.iter().enumerate() { cg.locals.insert(name.clone(), i); cg.next_local = cg.next_local.max(i + 1); }
        cg
    }
    fn local_count(&self) -> usize { self.next_local }
    fn get_local(&self, name: &str) -> Option<usize> { self.locals.get(name).copied() }
    fn alloc_local(&mut self, name: &str) -> usize { if let Some(&i) = self.locals.get(name) { i } else { let i = self.next_local; self.locals.insert(name.to_string(), i); self.next_local += 1; i } }

    fn block(&mut self, b: &Block, out: &mut Chunk) -> Result<()> { for s in &b.stmts { self.stmt(s, out)?; } Ok(()) }

    fn stmt(&mut self, s: &Stmt, out: &mut Chunk) -> Result<()> {
        match s {
            Stmt::Let(d) => { self.expr(&d.init, out)?; let idx = self.alloc_local(&d.name); out.code.push(Op::StoreLocal(idx)); }
            Stmt::Assign { name, value } => {
                self.expr(value, out)?; let idx = self.get_local(name).ok_or_else(|| anyhow!(format!("Értékadás előtt nincs változó: {}", name)))?;
                out.code.push(Op::StoreLocal(idx));
            }
            Stmt::Return(None) => { out.code.push(Op::PushVoid); out.code.push(Op::Return); }
            Stmt::Return(Some(e)) => { self.expr(e, out)?; out.code.push(Op::Return); }
            Stmt::If { cond, then_block, else_block } => {
                self.expr(cond, out)?; let jf = out.code.len(); out.code.push(Op::JumpIfFalse(usize::MAX));
                self.block(then_block, out)?;
                if let Some(else_b) = else_block {
                    let je = out.code.len(); out.code.push(Op::Jump(usize::MAX));
                    out.code[jf] = Op::JumpIfFalse(out.code.len());
                    self.block(else_b, out)?;
                    out.code[je] = Op::Jump(out.code.len());
                } else { out.code[jf] = Op::JumpIfFalse(out.code.len()); }
            }
            Stmt::While { cond, body } => {
                let start = out.code.len();
                self.expr(cond, out)?; let jf = out.code.len(); out.code.push(Op::JumpIfFalse(usize::MAX));
                self.loops.push(LoopCtx { start, breaks: Vec::new(), continues: Vec::new() });
                self.block(body, out)?;
                out.code.push(Op::Jump(start));
                let end = out.code.len();
                out.code[jf] = Op::JumpIfFalse(end);
                let lp = self.loops.pop().unwrap();
                for bpos in lp.breaks { out.code[bpos] = Op::Jump(end); }
for cpos in lp.continues { out.code[cpos] = Op::Jump(lp.start); }
            }
            Stmt::ForIn { var, iter, body } => {
                self.expr(iter, out)?; let arr_local = self.alloc_local("__for_arr"); out.code.push(Op::StoreLocal(arr_local));
                let idx_local = self.alloc_local("__for_i"); out.code.push(Op::PushInt(0)); out.code.push(Op::StoreLocal(idx_local));
                let start = out.code.len();
                out.code.push(Op::LoadLocal(idx_local));
                out.code.push(Op::LoadLocal(arr_local));
                out.code.push(Op::CallName("len".to_string(), 1));
                out.code.push(Op::Lt);
                let jf = out.code.len(); out.code.push(Op::JumpIfFalse(usize::MAX));
                self.loops.push(LoopCtx { start, breaks: Vec::new(), continues: Vec::new() });
                let v_local = self.alloc_local(var);
                out.code.push(Op::LoadLocal(arr_local));
                out.code.push(Op::LoadLocal(idx_local));
                out.code.push(Op::IndexGet);
                out.code.push(Op::StoreLocal(v_local));
                self.block(body, out)?;
                let cont_jump_pos = out.code.len();
                out.code.push(Op::LoadLocal(idx_local));
                out.code.push(Op::PushInt(1));
                out.code.push(Op::Add);
                out.code.push(Op::StoreLocal(idx_local));
                out.code.push(Op::Jump(start));
                let end = out.code.len();
                out.code[jf] = Op::JumpIfFalse(end);
                let lp = self.loops.pop().unwrap();
                for bpos in lp.breaks { out.code[bpos] = Op::Jump(end); }
for cpos in lp.continues { out.code[cpos] = Op::Jump(lp.start); }
            }
            Stmt::Break => {
                if let Some(lp) = self.loops.last_mut() { let pos = out.code.len(); out.code.push(Op::Jump(usize::MAX)); lp.breaks.push(pos); }
                else { return Err(anyhow!("break: nincs ciklusban")); }
            }
            Stmt::Continue => {
                if let Some(lp) = self.loops.last_mut() { let pos = out.code.len(); out.code.push(Op::Jump(usize::MAX)); lp.continues.push(pos); }
                else { return Err(anyhow!("continue: nincs ciklusban")); }
            }
            Stmt::Expr(e) => { self.expr(e, out)?; out.code.push(Op::Pop); }
        }
        Ok(())
    }

    fn expr(&mut self, e: &Expr, out: &mut Chunk) -> Result<()> {
        match e {
            Expr::Ident(name) => {
                if let Some(&idx) = self.locals.get(name) { out.code.push(Op::LoadLocal(idx)); }
                else { return Err(anyhow!(format!("Ismeretlen azonosító: {}", name))); }
            }
            Expr::Int(n) => out.code.push(Op::PushInt(*n)),
            Expr::Str(s) => out.code.push(Op::PushStr(s.clone())),
            Expr::Bool(b) => out.code.push(Op::PushBool(*b)),
            Expr::Array(elems) => { for el in elems { self.expr(el, out)?; } out.code.push(Op::MakeArray(elems.len())); }
            Expr::Index { target, index } => { self.expr(target, out)?; self.expr(index, out)?; out.code.push(Op::IndexGet); }
            Expr::Group(inner) => self.expr(inner, out)?,
            Expr::Binary { op, left, right } => {
                self.expr(left, out)?; self.expr(right, out)?;
                match op {
                    BinOp::Add => out.code.push(Op::Add),
                    BinOp::Sub => out.code.push(Op::Sub),
                    BinOp::Mul => out.code.push(Op::Mul),
                    BinOp::Div => out.code.push(Op::Div),
                    BinOp::Eq  => out.code.push(Op::Eq),
                    BinOp::Ne  => out.code.push(Op::Ne),
                    BinOp::Lt  => out.code.push(Op::Lt),
                    BinOp::Le  => out.code.push(Op::Le),
                    BinOp::Gt  => out.code.push(Op::Gt),
                    BinOp::Ge  => out.code.push(Op::Ge),
                }
            }
            Expr::Call { callee, args } => {
                let name = match &**callee { Expr::Ident(n) => n.clone(), _ => return Err(anyhow!("Csak név alapú hívás")) };
                for a in args { self.expr(a, out)?; }
                out.code.push(Op::CallName(name, args.len()));
            }
        }
        Ok(())
    }
}
#[allow(dead_code)]
impl Codegen {
    pub fn local_count(&self, fn_name: &str) -> usize {
        self.funcs.iter().find(|f| f.name == fn_name).map(|f| f.local_count).unwrap_or(0)
    }
}
