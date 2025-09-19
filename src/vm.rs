use std::collections::HashMap;
use anyhow::{anyhow, Result};
use crate::ir::*;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
    Array(Vec<Value>),
    Void,
}

pub struct VM {
    funcs: Vec<FunctionIR>,
    index: HashMap<String, usize>,
}

impl VM {
    pub fn new(p: ProgramIR) -> Self {
        let mut index = HashMap::new();
        for (i, f) in p.functions.iter().enumerate() { index.insert(f.name.clone(), i); }
        Self { funcs: p.functions, index }
    }

    pub fn run(&self, entry: &str) -> Result<Value> {
        let idx = *self.index.get(entry).ok_or_else(|| anyhow!(format!("Nincs ilyen függvény: {}", entry)))?;
        self.call(idx, Vec::new())
    }

    fn call(&self, idx: usize, args: Vec<Value>) -> Result<Value> {
        let f = &self.funcs[idx];
        if args.len() != f.arity { return Err(anyhow!("Arity mismatch")); }
        let mut stack: Vec<Value> = Vec::new();
        let mut locals: Vec<Value> = vec![Value::Void; f.local_count.max(args.len())];
        for i in 0..args.len() { locals[i] = args[i].clone(); }
        let mut ip: usize = 0;

        while ip < f.chunk.code.len() {
            match &f.chunk.code[ip] {
                Op::PushInt(n) => stack.push(Value::Int(*n)),
                Op::PushStr(s) => stack.push(Value::Str(s.clone())),
                Op::PushBool(b) => stack.push(Value::Bool(*b)),
                Op::PushVoid => stack.push(Value::Void),
                Op::LoadLocal(i) => stack.push(locals[*i].clone()),
                Op::StoreLocal(i) => {
                    let v = stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
                    if *i >= locals.len() { locals.resize(i+1, Value::Void); }
                    locals[*i] = v;
                }
                Op::MakeArray(n) => {
                    if stack.len() < *n { return Err(anyhow!("Stack underflow (MakeArray)")); }
                    let start = stack.len() - *n;
                    let mut v = stack.split_off(start);
                    // elemek balról jobbra kerüljenek a tömbbe
                    let arr = Value::Array(v.drain(..).collect());
                    stack.push(arr);
                }
                Op::IndexGet => {
                    let idx_v = stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
                    let tgt_v = stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
                    let i = match idx_v { Value::Int(k) => k as usize, _ => return Err(anyhow!("Index nem egész")) };
                    match tgt_v {
                        Value::Array(a) => {
                            if i >= a.len() { return Err(anyhow!("Index tartományon kívül")); }
                            stack.push(a[i].clone());
                        }
                        _ => return Err(anyhow!("Indexelés csak tömbön támogatott")),
                    }
                }
                Op::Add | Op::Sub | Op::Mul | Op::Div |
                Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge => {
                    let b = stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
                    let a = stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
                    stack.push(apply_binop(&a, &b, &f.chunk.code[ip])?);
                }
                Op::CallName(name, argc) => {
                    if name == "kiir" || name == "print" {
                        let mut args_buf = Vec::new();
                        for _ in 0..*argc { args_buf.push(stack.pop().unwrap()); }
                        args_buf.reverse();
                        let line = args_buf.iter().map(val_to_string).collect::<Vec<_>>().join(" ");
                        println!("{}", line);
                        stack.push(Value::Void);
                    } else if let Some(&callee_idx) = self.index.get(name) {
                        let mut call_args = Vec::new();
                        for _ in 0..*argc { call_args.push(stack.pop().unwrap()); }
                        call_args.reverse();
                        let ret = self.call(callee_idx, call_args)?;
                        stack.push(ret);
                    } else {
                        return Err(anyhow!(format!("Ismeretlen függvény: {}", name)));
                    }
                }
                Op::Pop => { stack.pop(); }
                Op::Jump(tgt) => { ip = *tgt; continue; }
                Op::JumpIfFalse(tgt) => {
                    let v = stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
                    if matches!(v, Value::Bool(false)) { ip = *tgt; continue; }
                }
                Op::Return => {
                    let v = stack.pop().unwrap_or(Value::Void);
                    return Ok(v);
                }
            }
            ip += 1;
        }
        Ok(Value::Void)
    }
}

fn val_to_string(v: &Value) -> String {
    match v {
        Value::Int(n) => n.to_string(),
        Value::Str(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Array(a) => {
            let inner = a.iter().map(val_to_string).collect::<Vec<_>>().join(", ");
            format!("[{}]", inner)
        }
        Value::Void => "()".to_string(),
    }
}

fn apply_binop(a: &Value, b: &Value, op: &Op) -> Result<Value> {
    use Value::*;
    Ok(match (op, a, b) {
        (Op::Add, Int(x), Int(y)) => Int(x + y),
        (Op::Sub, Int(x), Int(y)) => Int(x - y),
        (Op::Mul, Int(x), Int(y)) => Int(x * y),
        (Op::Div, Int(x), Int(y)) => Int(x / y),
        (Op::Eq,  Int(x), Int(y)) => Bool(x == y),
        (Op::Ne,  Int(x), Int(y)) => Bool(x != y),
        (Op::Lt,  Int(x), Int(y)) => Bool(x <  y),
        (Op::Le,  Int(x), Int(y)) => Bool(x <= y),
        (Op::Gt,  Int(x), Int(y)) => Bool(x >  y),
        (Op::Ge,  Int(x), Int(y)) => Bool(x >= y),
        _ => return Err(anyhow!("Nem támogatott művelet vagy típuspár")),
    })
}
