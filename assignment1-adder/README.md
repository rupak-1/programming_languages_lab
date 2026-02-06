# Week 1: Adder - A Simple Expression Compiler

## Overview
In this assignment, you'll implement a compiler for a small language called **Adder**, which supports 32-bit integers and three unary operations: `add1`, `sub1`, and `negate`. You'll build this from scratch to understand the complete compilation pipeline.

## Learning Objectives
- Understand the basic structure of a compiler
- Generate x86-64 assembly code from abstract syntax
- Learn about parsing, code generation, and linking
- Work with Rust's type system and pattern matching

## Setup

### Required Tools
1. **Rust and Cargo**: Install from https://www.rust-lang.org/tools/install
2. **NASM** (Netwide Assembler): 
   - macOS: `brew install nasm`
   - Linux: `sudo apt-get install nasm` or your package manager
   - Windows: Use WSL (Windows Subsystem for Linux)
3. **GCC/Clang**: For linking (usually pre-installed)

### System Requirements
- Must support x86-64 binaries
- Linux, macOS, or Windows with WSL
- Modern Macs with ARM can run x86-64 via Rosetta 2

### Project Creation
```bash
cargo new adder
cd adder
```

## The Adder Language

### Concrete Syntax
```
<expr> :=
  | <number>
  | (add1 <expr>)
  | (sub1 <expr>)
  | (negate <expr>)
```

Where `<number>` is a 32-bit signed integer.

### Abstract Syntax (Rust)
```rust
enum Expr {
    Num(i32),
    Add1(Box<Expr>),
    Sub1(Box<Expr>),
    Negate(Box<Expr>)
}
```

### Semantics
- **Numbers** evaluate to themselves
- **add1(e)** evaluates `e` and adds 1 to the result
- **sub1(e)** evaluates `e` and subtracts 1 from the result  
- **negate(e)** evaluates `e` and multiplies the result by -1

### Examples

**Example 1:**
```scheme
(add1 (sub1 5))
```
Result: `5`

**Example 2:**
```scheme
4
```
Result: `4`

**Example 3:**
```scheme
(negate (add1 3))
```
Result: `-4`

**Example 4:**
```scheme
(sub1 (sub1 (add1 73)))
```
Result: `72`

## Architecture Overview

Your compiler will consist of these components:

1. **Runtime** (`runtime/start.rs`): Rust program that calls compiled code
2. **Parser**: Converts text to S-expressions to AST
3. **Code Generator**: Converts AST to assembly instructions
4. **Build System**: Makefile to orchestrate compilation

### Compilation Goal
> "Compiling" an expression means generating assembly instructions that evaluate it and leave the answer in the `rax` register.

## Implementation Tasks

### Task 1: Create the Runtime (runtime/start.rs)

Create a Rust file that will serve as the entry point:

```rust
#[link(name = "our_code")]
extern "C" {
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here() -> i64;
}

fn main() {
    let i: i64 = unsafe {
        our_code_starts_here()
    };
    println!("{i}");
}
```

### Task 2: Implement the Parser

Add the `sexp` crate to `Cargo.toml`:
```toml
[dependencies]
sexp = "1.1.4"
```

Implement the parser in `src/main.rs`:
```rust
use sexp::*;
use sexp::Atom::*;

fn parse_expr(s: &Sexp) -> Expr {
    match s {
        Sexp::Atom(I(n)) => Expr::Num(i32::try_from(*n).unwrap()),
        Sexp::List(vec) => {
            match &vec[..] {
                [Sexp::Atom(S(op)), e] if op == "add1" => 
                    Expr::Add1(Box::new(parse_expr(e))),
                [Sexp::Atom(S(op)), e] if op == "sub1" => 
                    Expr::Sub1(Box::new(parse_expr(e))),
                // TODO: Add negate case
                _ => panic!("Invalid expression"),
            }
        },
        _ => panic!("Invalid expression"),
    }
}
```

### Task 3: Implement the Code Generator

```rust
fn compile_expr(e: &Expr) -> String {
    match e {
        Expr::Num(n) => format!("mov rax, {}", *n),
        Expr::Add1(subexpr) => compile_expr(subexpr) + "\nadd rax, 1",
        Expr::Sub1(subexpr) => compile_expr(subexpr) + "\nsub rax, 1",
        // TODO: Add negate case
    }
}
```

### Task 4: Create Main Function

```rust
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let in_name = &args[1];
    let out_name = &args[2];

    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    let expr = parse_expr(&parse(&in_contents).unwrap());
    let result = compile_expr(&expr);
    
    let asm_program = format!("
section .text
global our_code_starts_here
our_code_starts_here:
  {}
  ret
", result);

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}
```

### Task 5: Create Makefile

Create a `Makefile` (use tabs, not spaces for indentation):

For Linux:
```makefile
test/%.s: test/%.snek src/main.rs
	cargo run -- $< test/$*.s

test/%.run: test/%.s runtime/start.rs
	nasm -f elf64 test/$*.s -o runtime/our_code.o
	ar rcs runtime/libour_code.a runtime/our_code.o
	rustc -L runtime/ runtime/start.rs -o test/$*.run
```

For macOS:
```makefile
test/%.s: test/%.snek src/main.rs
	cargo run -- $< test/$*.s

test/%.run: test/%.s runtime/start.rs
	nasm -f macho64 test/$*.s -o runtime/our_code.o
	ar rcs runtime/libour_code.a runtime/our_code.o
	rustc -L runtime/ runtime/start.rs -o test/$*.run
```

## Testing

### Create Test Files

Create `test/` directory and add test files:

**test/37.snek:**
```scheme
37
```

**test/add.snek:**
```scheme
(add1 (add1 5))
```

**test/complex.snek:**
```scheme
(sub1 (sub1 (add1 73)))
```

**test/negate.snek:**
```scheme
(negate (add1 3))
```

### Running Tests

```bash
# Compile and run
make test/37.run
./test/37.run

# Check generated assembly
cat test/37.s
```

Expected output for `test/37.run`: `37`

## Deliverables

### Required Files
1. Complete Rust compiler in `src/main.rs`
2. Runtime in `runtime/start.rs`
3. Makefile
4. At least 10 test files in `test/` directory
5. `transcript.txt` showing your compiler working

### Transcript Requirements

Demonstrate your compiler working on at least 5 different examples:

```bash
# For each test:
cat test/example.snek
make test/example.run
cat test/example.s
./test/example.run
```

Copy the terminal output to `transcript.txt`.

## Grading Rubric

- **Parser (25%)**: Correctly parses all valid Adder programs
- **Code Generation (35%)**: Generates correct assembly for all operations
- **Testing (20%)**: Comprehensive test suite with edge cases
- **Documentation (10%)**: Clear code comments and README
- **Transcript (10%)**: Demonstrates working compiler

## Extension Challenges (Optional)

1. **Better Error Messages**: Add line/column numbers to parse errors
2. **Constant Folding**: Optimize `(add1 (sub1 5))` to just `5`
3. **Type Checking**: Reject programs at compile time when possible
4. **Pretty Printer**: Convert AST back to readable s-expressions

## Common Pitfalls

1. **Forgetting Box**: Recursive types need `Box<Expr>`
2. **Wrong Assembly Format**: Use `elf64` on Linux, `macho64` on macOS
3. **Tab vs Spaces**: Makefiles require tabs
4. **Integer Overflow**: Handle 32-bit boundaries properly

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [x86-64 Reference](https://web.stanford.edu/class/archive/cs/cs107/cs107.1196/guide/x86-64.html)
- [NASM Documentation](https://www.nasm.us/xdoc/2.15.05/html/nasmdoc0.html)
- [S-Expression Parser](https://crates.io/crates/sexp)

## Submission

Submit your entire project directory including:
- All source code
- Test files
- Makefile
- transcript.txt
- README.md (optional but recommended)

Good luck! This is the foundation for all future compiler assignments.
