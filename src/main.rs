mod token;
mod lexer;
mod ast;
mod parser;
mod ir;
mod codegen;
mod vm;

use std::fs;
use anyhow::Result;
use lexer::Lexer;
use parser::Parser;
use codegen::Codegen;
use vm::VM;

fn main() -> Result<()> {
    // --locale=hu|en, --file=path
    let mut locale = "hu".to_string();
    let mut file = None::<String>;
    for arg in std::env::args().skip(1) {
        if let Some(v) = arg.strip_prefix("--locale=") { locale = v.to_string(); continue; }
        if let Some(v) = arg.strip_prefix("--file=") { file = Some(v.to_string()); continue; }
    }
    let json = match locale.as_str() {
        "hu" => include_str!("../langpacks/hu.json"),
        "en" => include_str!("../langpacks/en.json"),
        other => panic!("ismeretlen locale: {}", other),
    };

    let lx = Lexer::from_locale_json(json)?;
    let src = if let Some(p) = file {
        fs::read_to_string(p)?
    } else {
        String::from(r#"
            osztaly Kutya {
                fuggveny ugat() { ha (igaz) { kiir("Vau"); } }
            }
            fuggveny fo() {
                legyen x = 1 + 2 * 3;
                kiir("osszeg:", x);
                vissza x;
            }
        "#)
    };

    let tokens = lx.lex(&src)?;
    let mut p = Parser::new(tokens);
    let program = p.parse_program()?;

    let cg = Codegen::new();
    let ir = cg.build(&program)?;

    let vm = VM::new(ir);
    let ret = vm.run("fo")?;
    println!("fo() -> {:?}", ret);
    Ok(())
}
