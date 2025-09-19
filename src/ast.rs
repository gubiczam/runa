#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Class(ClassDecl),
    Func(FuncDecl),
    Let(LetDecl),
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub methods: Vec<FuncDecl>,
}

#[derive(Debug, Clone)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct LetDecl {
    pub name: String,
    pub init: Expr,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetDecl),
    Assign { name: String, value: Expr },
    Return(Option<Expr>),
    If { cond: Expr, then_block: Block, else_block: Option<Block> },
    While { cond: Expr, body: Block },
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Ident(String),
    Int(i64),
    Str(String),
    Bool(bool),
    Array(Vec<Expr>),
    Index { target: Box<Expr>, index: Box<Expr> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Group(Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp { Add, Sub, Mul, Div, Eq, Ne, Lt, Le, Gt, Ge }
