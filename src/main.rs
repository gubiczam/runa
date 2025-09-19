use anyhow::{anyhow, Result};
use std::{env, fs};

mod token; mod lexer; mod ast; mod parser; mod ir; mod codegen; mod vm;

use lexer::Lexer;
use parser::Parser;
use codegen::Codegen;
use vm::VM;

fn main() -> Result<()> {
    // ---- args: --locale=<hu|en> --file=<path> ----
    let mut locale = String::from("en");
    let mut file: Option<String> = None;
    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--locale" => { i += 1; locale = args.get(i).cloned().ok_or_else(|| anyhow!("--locale needs value"))?; }
            "--file"   => { i += 1; file = Some(args.get(i).cloned().ok_or_else(|| anyhow!("--file needs value"))?); }
            x if x.starts_with("--locale=") => { locale = x["--locale=".len()..].to_string(); }
            x if x.starts_with("--file=")   => { file = Some(x["--file=".len()..].to_string()); }
            other => return Err(anyhow!(format!("Unknown arg: {}", other))),
        }
        i += 1;
    }

    // ---- langpack betöltés ----
    let lp_path = format!("langpacks/{}.json", locale);
    let lp_json = fs::read_to_string(&lp_path)
        .map_err(|e| anyhow!("Cannot read {}: {}", lp_path, e))?;
    let lexer = Lexer::from_locale_json(&lp_json)?;

    // ---- forrás beolvasása vagy demó ----
    let src = if let Some(p) = file {
        fs::read_to_string(&p).map_err(|e| anyhow!("Cannot read source file: {}", e))?
    } else {
        default_demo(&locale).to_string()
    };

    // ---- fordítási lánc ----
    let toks = lexer.lex(&src)?;
    let mut parser = Parser::new(toks);
    let program = parser.parse_program()?;

    let ir = Codegen::new().build(&program)?;
    let vm = VM::new(ir);

    // ---- belépési pont ----
    let entries = if locale == "hu" { vec!["fo", "main"] } else { vec!["main", "fo"] };
    for e in entries {
        if let Ok(val) = vm.run(e) {
            println!("{}() -> {:?}", e, val);
            return Ok(());
        }
    }
    Err(anyhow!("No entry function found (expected: main/fo)"))
}

// Kis beágyazott demó csak fallbacknek, lokálé szerint
fn default_demo(locale: &str) -> &'static str {
    if locale == "hu" {
        r#"
fuggveny fo() {
  legyen a = [1,2,3];
  kiir("len(a)=", len(a));
  vissza a[0] + a[1] + a[2];
}
"#
    } else {
        r#"
fn main() {
  let a = [1,2,3];
  print("len(a) =", len(a));
  return a[0] + a[1] + a[2];
}
"#
    }
}