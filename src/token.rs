#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Kulcsszavak
    KwClass, KwFn, KwIf, KwElse, KwReturn, KwLet, KwVar,
    KwWhile, KwFor, KwIn, KwTrue, KwFalse, KwVoid,
    // Azonosítók/literálok
    Ident(String), Int(i64), Str(String),
    // Jelek/operátorok
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Dot, Colon, Semicolon, Arrow,
    Plus, Minus, Star, Slash, Percent,
    Assign, Eq, Ne, Lt, Le, Gt, Ge,
    AndAnd, OrOr, Not,
    Eof,
}
