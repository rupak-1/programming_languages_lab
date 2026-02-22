# Adder Compiler

A simple compiler for the Adder language that compiles arithmetic expressions to x86-64 assembly code.

## What It Does

The Adder language supports 32-bit integers and three unary operations:
- `add1` - adds 1 to a number
- `sub1` - subtracts 1 from a number  
- `negate` - multiplies a number by -1

The compiler takes Adder programs written as S-expressions and generates x86-64 assembly code that evaluates the expression and leaves the result in the `rax` register.

## Language Syntax

```
<expr> :=
  | <number>
  | (add1 <expr>)
  | (sub1 <expr>)
  | (negate <expr>)
```

Examples:
- `37` evaluates to 37
- `(add1 5)` evaluates to 6
- `(sub1 (add1 5))` evaluates to 5
- `(negate (add1 3))` evaluates to -4

## Project Structure

- `src/main.rs` - Main compiler implementation (parser and code generator)
- `runtime/start.rs` - Runtime entry point that calls the compiled code
- `Makefile` - Build system for compiling and running tests
- `test/` - Test files written in Adder syntax (.snek files)

## How It Works

1. **Parser**: Reads S-expression input and converts it to an Abstract Syntax Tree (AST)
2. **Code Generator**: Traverses the AST and generates x86-64 assembly instructions
3. **Runtime**: The generated assembly is linked with a Rust runtime that calls the compiled code and prints the result

## Building and Running

### Prerequisites

- Rust and Cargo
- NASM assembler (`brew install nasm` on macOS)
- GCC/Clang for linking

### Compile a test file

```bash
make test/basic.run
./test/basic.run
```

This will:
1. Parse `test/basic.snek`
2. Generate `test/basic.s` (assembly)
3. Assemble and link to create `test/basic.run`
4. Run the executable

### Run all tests

```bash
make test
```

### Clean build artifacts

```bash
make clean
```

## Implementation Details

The AST is represented as:

```rust
enum Expr {
    Num(i32),
    Add1(Box<Expr>),
    Sub1(Box<Expr>),
    Negate(Box<Expr>)
}
```

Code generation works recursively:
- Numbers: `mov rax, <value>`
- Operations: compile subexpression first, then apply operation (`add rax, 1`, `sub rax, 1`, `imul rax, -1`)

The generated assembly follows this template:

```asm
section .text
global our_code_starts_here
our_code_starts_here:
  <generated instructions>
  ret
```

## Notes

- Uses the `sexp` crate for parsing S-expressions
- Assembly format is `macho64` for macOS (change to `elf64` for Linux)
- The runtime uses FFI to call the generated assembly function
- All operations work with 32-bit signed integers
