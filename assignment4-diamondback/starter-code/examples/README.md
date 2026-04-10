# Example Diamondback programs

Run the compiler and link as in the course `Makefile`, then execute with an optional numeric argument for `input`:

```bash
cargo run -- examples/<name>.snek out.s
nasm -f elf64 out.s -o runtime/our_code.o   # use macho64 on macOS x86_64 if applicable
# … link with runtime/start.rs per Makefile …
./program.run [input_integer]
```

| File | Demonstrates |
|------|----------------|
| `01_literals.snek` | Numeric and boolean literals |
| `02_comparisons.snek` | `<` `>` `<=` `>=` `=` on numbers |
| `03_isnum_isbool.snek` | `isnum` / `isbool` |
| `04_if.snek` | Conditional |
| `05_block.snek` | `block` sequencing |
| `06_loop_break.snek` | `loop` / `break` (README counter) |
| `07_set.snek` | `set!` mutation |
| `08_input.snek` | `input` (pass value as CLI arg to the runtime) |
| `09_nested_if.snek` | Nested `if` |
| `10_mixed.snek` | Boolean and numeric expressions together |
| `11_factorial.snek` | Recursive factorial function |
| `12_fibonacci.snek` | Recursive fibonacci function |
| `13_mutual_recursion.snek` | Two functions calling each other |
| `error_invalid_add.snek` | **Runtime error**: `(+ true 5)` → `invalid argument` |
