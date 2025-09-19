use anyhow::{anyhow, Result};
use logos::Logos;
use std::collections::HashMap;
use crate::token::TokenKind;

#[derive(Logos, Debug, PartialEq)]
enum RawTok {
    #[regex(r"[ \t\r\n\f]+", logos::skip)]
    Whitespace,
    #[regex(r#"//[^\n]*"#, logos::skip)]
    LineComment,

    #[regex(r#""([^"\\]|\\.)*""#)]
    Str,
    #[regex(r"[0-9]+")]
    Int,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Word,

    #[token("(")] LParen,   #[token(")")] RParen,
    #[token("{")] LBrace,   #[token("}")] RBrace,
    #[token("[")] LBracket, #[token("]")] RBracket,
    #[token(",")] Comma,    #[token(".")] Dot,
    #[token(":")] Colon,    #[token(";")] Semicolon,
    #[token("->")] Arrow,
    #[token("+")] Plus,     #[token("-")] Minus,
    #[token("*")] Star,     #[token("/")] Slash,
    #[token("%")] Percent,
    #[token("=")] Assign,   #[token("==")] Eq,  #[token("!=")] Ne,
    #[token("<")] Lt,       #[token("<=")] Le,
    #[token(">")] Gt,       #[token(">=")] Ge,
    #[token("&&")] AndAnd,  #[token("||")] OrOr, #[token("!")] Not,
}

pub struct Lexer { locale: HashMap<String, TokenKind> }

impl Lexer {
    pub fn from_locale_json(json: &str) -> Result<Self> {
        let raw: HashMap<String, String> = serde_json::from_str(json)?;
        let mut map = HashMap::new();
        for (k, v) in raw {
            let tk = match v.as_str() {
                "KwClass" => TokenKind::KwClass,
                "KwFn" => TokenKind::KwFn,
                "KwIf" => TokenKind::KwIf,
                "KwElse" => TokenKind::KwElse,
                "KwReturn" => TokenKind::KwReturn,
                "KwLet" => TokenKind::KwLet,
                "KwVar" => TokenKind::KwVar,
                "KwWhile" => TokenKind::KwWhile,
                "KwFor" => TokenKind::KwFor,
                "KwIn" => TokenKind::KwIn,
                "KwBreak" => TokenKind::KwBreak,
                "KwContinue" => TokenKind::KwContinue,
                "KwTrue" => TokenKind::KwTrue,
                "KwFalse" => TokenKind::KwFalse,
                "KwVoid" => TokenKind::KwVoid,
                other => return Err(anyhow!(format!("ismeretlen kulcsszó azonosító: {}", other))),
            };
            map.insert(k, tk);
        }
        Ok(Self { locale: map })
    }

    pub fn lex(&self, src: &str) -> Result<Vec<TokenKind>> {
        let mut out = Vec::new();
        let mut lexer = RawTok::lexer(src);

        while let Some(res) = lexer.next() {
            match res {
                Ok(tok) => match tok {
                    RawTok::Whitespace | RawTok::LineComment => {}
                    RawTok::Str => {
                        let slice = lexer.slice();
                        let unq = unquote(slice)?;
                        out.push(TokenKind::Str(unq));
                    }
                    RawTok::Int => {
                        let n: i64 = lexer.slice().parse()?;
                        out.push(TokenKind::Int(n));
                    }
                    RawTok::Word => {
                        let w = lexer.slice().to_string();
                        if let Some(kw) = self.locale.get(&w) { out.push(kw.clone()); }
                        else { out.push(TokenKind::Ident(w)); }
                    }
                    RawTok::LParen => out.push(TokenKind::LParen),
                    RawTok::RParen => out.push(TokenKind::RParen),
                    RawTok::LBrace => out.push(TokenKind::LBrace),
                    RawTok::RBrace => out.push(TokenKind::RBrace),
                    RawTok::LBracket => out.push(TokenKind::LBracket),
                    RawTok::RBracket => out.push(TokenKind::RBracket),
                    RawTok::Comma => out.push(TokenKind::Comma),
                    RawTok::Dot => out.push(TokenKind::Dot),
                    RawTok::Colon => out.push(TokenKind::Colon),
                    RawTok::Semicolon => out.push(TokenKind::Semicolon),
                    RawTok::Arrow => out.push(TokenKind::Arrow),
                    RawTok::Plus => out.push(TokenKind::Plus),
                    RawTok::Minus => out.push(TokenKind::Minus),
                    RawTok::Star => out.push(TokenKind::Star),
                    RawTok::Slash => out.push(TokenKind::Slash),
                    RawTok::Percent => out.push(TokenKind::Percent),
                    RawTok::Assign => out.push(TokenKind::Assign),
                    RawTok::Eq => out.push(TokenKind::Eq),
                    RawTok::Ne => out.push(TokenKind::Ne),
                    RawTok::Lt => out.push(TokenKind::Lt),
                    RawTok::Le => out.push(TokenKind::Le),
                    RawTok::Gt => out.push(TokenKind::Gt),
                    RawTok::Ge => out.push(TokenKind::Ge),
                    RawTok::AndAnd => out.push(TokenKind::AndAnd),
                    RawTok::OrOr => out.push(TokenKind::OrOr),
                    RawTok::Not => out.push(TokenKind::Not),
                },
                Err(()) => { return Err(anyhow!(format!("lexikai hiba: poz {}", lexer.span().start))); }
            }
        }

        out.push(TokenKind::Eof);
        Ok(out)
    }
}

fn unquote(s: &str) -> Result<String> {
    let bytes = s.as_bytes();
    if bytes.len() < 2 { return Err(anyhow!("rossz string literal")); }
    let inner = &s[1..s.len()-1];
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(n) = chars.next() {
                match n {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    '\\' => out.push('\\'),
                    '"' => out.push('"'),
                    other => return Err(anyhow!(format!("ismeretlen escape: \\{}", other))),
                }
            } else { return Err(anyhow!("befejezetlen escape")); }
        } else { out.push(c); }
    }
    Ok(out)
}
