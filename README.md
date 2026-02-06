# CSCI282L-2026spring

This package contains four complete, independent programming assignments for teaching programming language lab. Each assignment builds progressively more sophisticated compiler features. This package is builded based on [UCSD CSE131/231](https://ucsd-compilers-s23.github.io/index.html). 

## 📦 Package Structure

```
compiler-assignments/
├── README.md (this file)
├── assignment1-adder/
│   ├── README.md                  # Assignment instructions
│   ├── starter-code/              # Student starting point
│   │   ├── src/main.rs
│   │   ├── runtime/start.rs
│   │   ├── Cargo.toml
│   │   ├── Makefile
│   │   └── test/
│   └── solution/                  # Reference solution
│       ├── src/main.rs
│       ├── runtime/start.rs
│       ├── Cargo.toml
│       ├── Makefile
│       └── test/
├── assignment2-boa/               # Same structure
├── assignment3-cobra/             # Same structure
└── assignment4-diamondback/       # Same structure
```

## 🎯 Assignment Overview

### Assignment 1: Adder (Week 1)
**Estimated Time:** 8-12 hours  
**Topics:** Basic compiler structure, parsing, code generation

Build a compiler for simple arithmetic expressions:
- Numbers (32-bit integers)
- Unary operations: `add1`, `sub1`, `negate`
- S-expression parsing
- x86-64 assembly generation

**Example Programs:**
```scheme
37                      ; Result: 37
(add1 5)               ; Result: 6
(negate (add1 3))      ; Result: -4
```

### Assignment 2: Boa (Week 2)  
**Estimated Time:** 12-16 hours  
**Topics:** Variables, binary operators, stack management

Extend the compiler with:
- `let` bindings with multiple variables
- Variable lookup and environments  
- Binary operators: `+`, `-`, `*`
- Stack-based variable storage

**Example Programs:**
```scheme
(let ((x 5)) x)                          ; Result: 5
(let ((x 5) (y 10)) (+ x y))           ; Result: 15
(let ((x 10) (y (+ x 5))) (* x y))     ; Result: 150
```

### Assignment 3: Cobra (Week 3)
**Estimated Time:** 16-20 hours  
**Topics:** Dynamic typing, control flow, loops

Add dynamic types and control structures:
- Boolean values with tag bits
- Comparison operators: `<`, `>`, `=`, etc.
- Conditional expressions: `if`
- Loops: `loop`, `break`
- Variable mutation: `set!`
- Runtime type checking

**Example Programs:**
```scheme
(if (< 3 5) 100 200)                    ; Result: 100

(let ((x 0))
  (loop
    (if (= x 10)
      (break x)
      (set! x (+ x 1)))))                ; Result: 10
```

### Assignment 4: Diamondback (Week 4)
**Estimated Time:** 20-24 hours  
**Topics:** Functions, recursion, calling conventions

Complete the compiler with:
- Function definitions
- Function calls with multiple arguments
- Stack frame management
- Recursive functions
- x86-64 calling conventions

**Example Programs:**
```scheme
(fun (factorial n)
  (if (= n 1)
    1
    (* n (factorial (- n 1)))))

(factorial 5)                            ; Result: 120
```

## 🛠️ Prerequisites

### Required Software
- **Rust** (1.65+): https://rustup.rs/
- **NASM**: Netwide Assembler
  - macOS: `brew install nasm`
  - Ubuntu/Debian: `sudo apt-get install nasm`
- **GCC or Clang**: For linking (usually pre-installed)

### Supported Platforms
- Linux (x86-64)
- macOS (Intel or Apple Silicon with Rosetta)
- Windows with WSL (Windows Subsystem for Linux)

## 🚀 Getting Started

### For Students

1. **Start with Assignment 1:**
   ```bash
   cd assignment1-adder/starter-code
   cat ../README.md  # Read instructions
   ```

2. **Build and test:**
   ```bash
   cargo build
   make test/example.run
   ./test/example.run
   ```

3. **Implement the compiler** following the instructions in README.md

4. **Check your solution** against the reference in `solution/` folder

### For Instructors

- **Starter code** provides scaffolding for students
- **Solution code** includes complete reference implementation  
- **README.md** in each assignment has detailed instructions
- Distribute only the `starter-code` folder to students
- Use `solution` for grading and office hours

## 📝 Assignment Workflow

Each assignment follows this pattern:

1. **Read** the README.md thoroughly
2. **Examine** the starter code structure
3. **Implement** the required features
4. **Test** with provided and custom test cases
5. **Compare** with solution (after attempting yourself!)

## 🧪 Testing

Each assignment includes sample tests in `test/` directory.

**Running tests:**
```bash
# Single test
make test/example.run
./test/example.run

# View generated assembly
cat test/example.s
```

**Students should create:**
- At least 10 tests for Assignment 1
- At least 15 tests for Assignment 2
- At least 20 tests for Assignment 3
- At least 25 tests for Assignment 4

## 📚 Additional Resources

### Learning Materials
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [x86-64 Guide](https://web.stanford.edu/class/cs107/guide/x86-64.html) - Assembly reference
- [NASM Documentation](https://www.nasm.us/xdoc/2.15.05/html/nasmdoc0.html)

### Course Materials
Based on UCSD CSE 131/231 compiler construction course:
- https://ucsd-compilers-s23.github.io/

## 🎓 Grading Guidelines

### Suggested Point Distribution (per assignment)

| Component | Points |
|-----------|--------|
| Correctness (passes tests) | 40 |
| Code Quality | 25 |
| Testing (comprehensive suite) | 20 |
| Documentation | 15 |

### Assessment Criteria

**Correctness:**
- Passes all provided test cases
- Handles edge cases properly
- Appropriate error handling

**Code Quality:**
- Clean, readable code
- Proper abstractions
- Good variable/function names  
- Helpful comments

**Testing:**
- Comprehensive test coverage
- Tests edge cases
- Well-documented test cases

**Documentation:**
- README explaining design decisions
- Code comments for complex logic
- Examples demonstrating features

## 🔧 Common Issues

### Setup Problems

**NASM not found:**
```bash
# macOS
brew install nasm

# Linux
sudo apt-get install nasm
```

**Rust compilation errors:**
```bash
rustup update stable
```

### Runtime Issues

**Segmentation fault:**
- Check stack alignment (must be 16-byte aligned)
- Verify stack offset calculations
- Check rbp/rsp management

**Wrong output values:**
- Verify value tagging/untagging (Assignments 3-4)
- Check operator precedence
- Ensure environment is properly maintained

## 📖 Academic Integrity

### For Students
- Attempt assignments independently before consulting solutions
- Use solutions for learning, not copying
- Cite any code or ideas from external sources

### For Instructors  
- Solutions are provided for educational purposes
- Consider modifying test cases each semester
- Solutions help with grading and office hours

## 🤝 Acknowledgments

These assignments are based on the excellent compiler construction course at UCSD taught by Professor Joe Politz and colleagues. The incremental approach and snake-themed language names are inspired by their curriculum.

Additional thanks to:
- Max New and Ben Lerner (Northeastern University)
- The Rust community
- All students who have taken and improved these courses

## 📧 Support

For questions:
- Consult README.md in each assignment
- Post on course discussion forum
- Attend office hours
- Review solution code (after attempting!)

## 📄 License

Educational use only. Based on publicly available course materials.

---

**Good luck building your compilers!** 🐍🔧

Each assignment is a stepping stone to understanding how programming languages work at the lowest level.
