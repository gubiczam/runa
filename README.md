# Runa — multilingual keyword programming language (MVP)

Runa is a small programming language with **localized keywords** (Hungarian/English) and its own **compiler pipeline** (lexer → parser → bytecode IR) and **stack-based VM**.

- Keywords are loaded from `langpacks/*.json`.
- Source code stays readable in different natural languages while the compiler stays language-agnostic.

---

## Status

- **Localization:** `--locale=hu` or `--locale=en`
- **Syntax:** `class/osztaly`, `fn/fuggveny`, blocks, `let/legyen`, `if/ha`, `else/kulonben`, `while/amig`, `return/vissza`
- **Statements:** declaration `let/legyen`, assignment `x = expr;`
- **Expressions:** int, string, bool, arrays `[1,2,3]`, indexing `a[0]`, calls `foo(…)`
- **Operators:** `+ - * /`, `== != < <= > >=`
- **Built-ins:** `print/kiir`, `len(x)`, `push(arr, value)`
- **Not yet:** type checker, objects/fields, modules, richer stdlib

---

## Quick start

```bash
# macOS prerequisites:
xcode-select --install
curl https://sh.rustup.rs -sSf | sh -s -- -y
source "$HOME/.cargo/env"

# build & run
cargo build
cargo run -- --locale=hu --file=demo.hu
cargo run -- --locale=en --file=demo.en
```

File extension suggestion: **`.rn`**

---

## Examples

### English

```text
fn add(a, b) { return a + b; }

fn main() {
  let a = [10, 20];
  print("len(a) =", len(a));
  a = push(a, 99);
  print("a =", a, "last =", a[2]);
  if (a[2] == 99) { print("ok"); } else { print("fail"); }
  return add(a[0], a[1]);
}
```

Run:
```bash
cargo run -- --locale=en --file=example.en.rn
```

Expected output:
```
len(a) = 2
a = [10, 20, 99] last = 99
ok
main() -> Int(30)
```

### Hungarian (Magyar)

```text
fuggveny osszead(a, b) { vissza a + b; }

fuggveny fo() {
  legyen a = [1, 2];
  kiir("len(a)=", len(a));
  a = push(a, 99);
  kiir("a=", a, "utolso=", a[2]);
  ha (a[2] == 99) { kiir("ok"); } kulonben { kiir("bukta"); }
  vissza osszead(a[0], a[1]);
}
```

Run:
```bash
cargo run -- --locale=hu --file=peldak.hu.rn
```

Expected output:
```
len(a)= 2
a= [1, 2, 99] utolso= 99
ok
fo() -> Int(3)
```

---

## Project layout

```
/src
  token.rs     # token definitions
  lexer.rs     # Logos-based lexer + keyword i18n
  ast.rs       # AST types
  parser.rs    # recursive-descent parser
  ir.rs        # simple bytecode ops
  codegen.rs   # AST -> IR
  vm.rs        # stack VM interpreter
/langpacks
  hu.json
  en.json
```

---

## CLI

```
cargo run -- --locale=<hu|en> --file=path/to/source.rn
```

If `--file` is omitted, an embedded demo is used.

---

## Contributing workflow

- `main` keeps stable builds.
- Develop on feature branches: `feat/...`, `fix/...`, `docs/...`.
- Open PRs; prefer squash merges.

---

## Roadmap

- Type checker and diagnostics
- `for-in`, `break/continue`
- Array/String APIs beyond `len/push`
- Module/import system
- Formatter and LSP support
- WASM or native backend via LLVM

---

## License

TBD.
