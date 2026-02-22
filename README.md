# CSCI282L Compiler Assignments

A collection of compiler construction assignments building progressively more sophisticated language features. Based on UCSD CSE131/231 course materials.

## Project Structure

```
CSCI282L/
├── README.md
├── assignment1-adder/        # Simple arithmetic expressions
├── assignment2-boa/          # Variables and binary operators
├── assignment3-cobra/         # Control flow and dynamic types
└── assignment4-diamondback/   # Functions and recursion
```

Each assignment directory contains:
- `src/main.rs` - Compiler implementation
- `runtime/start.rs` - Runtime entry point
- `Makefile` - Build system
- `test/` - Test files
- `README.md` - Assignment-specific documentation

## Assignments

### Assignment 1: Adder

Simple arithmetic compiler supporting:
- 32-bit integers
- Unary operations: `add1`, `sub1`, `negate`
- S-expression parsing
- x86-64 assembly generation

Example:
```scheme
(negate (add1 3))  ; Result: -4
```

### Assignment 2: Boa

Extends Adder with:
- `let` bindings with multiple variables
- Variable lookup and environments
- Binary operators: `+`, `-`, `*`
- Stack-based variable storage

Example:
```scheme
(let ((x 5) (y 10)) (+ x y))  ; Result: 15
```

### Assignment 3: Cobra

Adds dynamic types and control flow:
- Boolean values with tag bits
- Comparison operators: `<`, `>`, `=`
- Conditional expressions: `if`
- Loops: `loop`, `break`
- Variable mutation: `set!`
- Runtime type checking

Example:
```scheme
(let ((x 0))
  (loop
    (if (= x 10)
      (break x)
      (set! x (+ x 1)))))  ; Result: 10
```

### Assignment 4: Diamondback

Complete compiler with:
- Function definitions
- Function calls with multiple arguments
- Stack frame management
- Recursive functions
- x86-64 calling conventions

Example:
```scheme
(fun (factorial n)
  (if (= n 1)
    1
    (* n (factorial (- n 1)))))

(factorial 5)  ; Result: 120
```

## Setup

### Requirements

- Rust (1.65+): https://rustup.rs/
- NASM assembler:
  - macOS: `brew install nasm`
  - Linux: `sudo apt-get install nasm`
- GCC or Clang for linking

### Platform Support

- Linux (x86-64)
- macOS (Intel or Apple Silicon with Rosetta)
- Windows with WSL

## Usage

### Building and Testing

Navigate to an assignment directory:

```bash
cd assignment1-adder
make test/basic.run
./test/basic.run
```

### Running All Tests

```bash
make test
```

### Viewing Generated Assembly

```bash
cat test/example.s
```

## Implementation Notes

Each assignment builds on the previous one:

1. **Adder**: Establishes basic compiler structure (parser, AST, code generator)
2. **Boa**: Introduces variable management and stack operations
3. **Cobra**: Adds type tagging, control flow, and runtime checks
4. **Diamondback**: Implements function calls and calling conventions

The compilation pipeline:
1. Parse S-expressions into AST
2. Generate x86-64 assembly from AST
3. Assemble and link with runtime
4. Execute and print result

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [x86-64 Guide](https://web.stanford.edu/class/cs107/guide/x86-64.html)
- [NASM Documentation](https://www.nasm.us/xdoc/2.15.05/html/nasmdoc0.html)
- [UCSD CSE 131/231](https://ucsd-compilers-s23.github.io/)

## Common Issues

**NASM not found:**
```bash
brew install nasm  # macOS
sudo apt-get install nasm  # Linux
```

**Segmentation faults:**
- Check stack alignment (16-byte aligned)
- Verify stack offset calculations
- Review rbp/rsp management

**Wrong output values:**
- Verify value tagging/untagging (Cobra/Diamondback)
- Check operator precedence
- Ensure environment is properly maintained

## Notes

Based on UCSD CSE 131/231 compiler construction course materials. Each assignment is designed to be completed independently, building understanding of compiler internals incrementally.
