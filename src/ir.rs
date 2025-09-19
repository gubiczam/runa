#[derive(Debug, Clone)]
pub enum Op {
    PushInt(i64),
    PushStr(String),
    PushBool(bool),
    PushVoid,
    LoadLocal(usize),
    StoreLocal(usize),
    Add, Sub, Mul, Div,
    Eq, Ne, Lt, Le, Gt, Ge,
    MakeArray(usize),
    IndexGet,
    CallName(String, usize),
    Pop,
    Jump(usize),
    JumpIfFalse(usize),
    Return,
}

#[derive(Debug, Clone)]
pub struct Chunk { pub code: Vec<Op> }
impl Chunk { pub fn new() -> Self { Self { code: Vec::new() } } }

pub struct FunctionIR {
    pub name: String,
    pub arity: usize,
    pub local_count: usize,
    pub chunk: Chunk,
}

pub struct ProgramIR {
    pub functions: Vec<FunctionIR>,
}
