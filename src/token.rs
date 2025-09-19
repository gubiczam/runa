#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    KwClass, KwFn, KwIf, KwElse, KwReturn, KwLet, KwVar,
    KwWhile, KwFor, KwIn, KwBreak, KwContinue,
    KwTrue, KwFalse, KwVoid,
    Ident(String), Int(i64), Str(String),
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Dot, Colon, Semicolon, Arrow,
    Plus, Minus, Star, Slash, Percent,
    Assign, Eq, Ne, Lt, Le, Gt, Ge,
    AndAnd, OrOr, Not,
    Eof,
}
