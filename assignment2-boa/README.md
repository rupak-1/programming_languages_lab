# Week 2: Boa - Variables and Binary Operators

## Overview
In this assignment, you'll extend your compiler to support **Boa** (Binary Operators And variables), adding `let` bindings and binary arithmetic/comparison operators. This introduces the stack for variable storage and the need for an environment to track variable locations.

## Learning Objectives
- Implement variable binding and lookup
- Generate code that uses the stack
- Handle binary operations
- Understand environment/symbol table management
- Learn about register allocation basics

## Building on Week 1
You should start from your Adder compiler or use the provided reference implementation. The key new concepts are:
- **Variables**: Store values on the stack
- **Let bindings**: Create new scope with variables
- **Binary operators**: Operations on two values

## The Boa Language

### Concrete Syntax
```
<expr> :=
  | <number>
  | <identifier>
  | (let ((<identifier> <expr>)+) <expr>)
  | (add1 <expr>)
  | (sub1 <expr>)
  | (negate <expr>)
  | (+ <expr> <expr>)
  | (- <expr> <expr>)
  | (* <expr> <expr>)

<identifier> := [a-zA-Z][a-zA-Z0-9]*  (but not reserved words)
```

Reserved words: `let`, `add1`, `sub1`, `negate`

### Abstract Syntax (Rust)
```rust
enum Expr {
    Num(i32),
    Var(String),
    Let(Vec<(String, Expr)>, Box<Expr>),  // bindings, body
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}

enum UnOp {
    Add1,
    Sub1,
    Negate,
}

enum BinOp {
    Plus,
    Minus,
    Times,
}
```

### Semantics

**Let bindings**: Evaluate binding expressions left-to-right, storing each result before evaluating the next. Each binding is visible in subsequent bindings and the body.

```scheme
(let ((x 5)
      (y (+ x 1)))  ; x is visible here
  (* x y))          ; both x and y visible here
; Result: 30
```

**Variables**: Look up the most recent binding of the identifier.

**Binary operators**: Evaluate both operands (left-to-right), then apply the operation.

### Examples

**Example 1:**
```scheme
(let ((x 5)) (+ x 1))
```
Result: `6`

**Example 2:**
```scheme
(let ((x 5)
      (y 10))
  (+ x y))
```
Result: `15`

**Example 3:**
```scheme
(let ((x 10)
      (y (+ x 5)))
  (* x y))
```
Result: `150`

**Example 4:**
```scheme
(let ((a 1)
      (b 2))
  (let ((c (+ a b)))
    (* c c)))
```
Result: `9`

## Implementation Strategy

### The Stack
The x86-64 stack grows downward (toward lower addresses). We'll use negative offsets from `rsp` to store local variables:

```
        Higher addresses
        ---------------
rsp-8   | variable 1  |
        ---------------
rsp-16  | variable 2  |
        ---------------
rsp-24  | variable 3  |
        ---------------
        Lower addresses
```

### Environment
Track variable locations using a HashMap:
```rust
use std::collections::HashMap;

type Env = HashMap<String, i32>;  // variable name -> stack offset
```

### Stack Offset Calculation
- Start at offset 8 (first variable at rsp-8)
- Increment by 8 for each new variable
- Pass current offset through compilation

## Implementation Tasks

### Task 1: Update AST Definition

```rust
#[derive(Debug, Clone)]
enum Expr {
    Num(i32),
    Var(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
enum UnOp {
    Add1,
    Sub1,
    Negate,
}

#[derive(Debug, Clone)]
enum BinOp {
    Plus,
    Minus,
    Times,
}
```

### Task 2: Extend Parser

```rust
fn parse_expr(s: &Sexp) -> Expr {
    match s {
        Sexp::Atom(I(n)) => Expr::Num(i32::try_from(*n).unwrap()),
        
        Sexp::Atom(S(name)) => {
            if name == "let" || name == "add1" || name == "sub1" || name == "negate" {
                panic!("Invalid use of keyword as identifier: {}", name);
            }
            Expr::Var(name.to_string())
        }
        
        Sexp::List(vec) => match &vec[..] {
            // Let expression
            [Sexp::Atom(S(op)), Sexp::List(bindings), body] if op == "let" => {
                let parsed_bindings = bindings.iter().map(|binding| {
                    match binding {
                        Sexp::List(pair) => match &pair[..] {
                            [Sexp::Atom(S(name)), expr] => {
                                (name.to_string(), parse_expr(expr))
                            }
                            _ => panic!("Invalid binding: {:?}", pair),
                        },
                        _ => panic!("Invalid binding: {:?}", binding),
                    }
                }).collect();
                Expr::Let(parsed_bindings, Box::new(parse_expr(body)))
            }
            
            // Unary operations
            [Sexp::Atom(S(op)), e] if op == "add1" => 
                Expr::UnOp(UnOp::Add1, Box::new(parse_expr(e))),
            [Sexp::Atom(S(op)), e] if op == "sub1" => 
                Expr::UnOp(UnOp::Sub1, Box::new(parse_expr(e))),
            [Sexp::Atom(S(op)), e] if op == "negate" => 
                Expr::UnOp(UnOp::Negate, Box::new(parse_expr(e))),
            
            // Binary operations
            [Sexp::Atom(S(op)), e1, e2] if op == "+" => 
                Expr::BinOp(BinOp::Plus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), e1, e2] if op == "-" => 
                Expr::BinOp(BinOp::Minus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            [Sexp::Atom(S(op)), e1, e2] if op == "*" => 
                Expr::BinOp(BinOp::Times, Box::new(parse_expr(e1)), Box::new(parse_expr(e2))),
            
            _ => panic!("Invalid expression: {:?}", vec),
        },
        
        _ => panic!("Invalid expression: {:?}", s),
    }
}
```

### Task 3: Implement Code Generation

```rust
use std::collections::HashMap;

fn compile_expr(e: &Expr, env: &HashMap<String, i32>, stack_offset: i32) -> String {
    match e {
        Expr::Num(n) => format!("mov rax, {}", n),
        
        Expr::Var(name) => {
            // Look up variable in environment
            match env.get(name) {
                Some(offset) => format!("mov rax, [rsp - {}]", offset),
                None => panic!("Unbound variable: {}", name),
            }
        }
        
        Expr::Let(bindings, body) => {
            let mut instrs = Vec::new();
            let mut new_env = env.clone();
            let mut current_offset = stack_offset;
            
            for (name, expr) in bindings {
                // Check for duplicate bindings
                if bindings.iter().filter(|(n, _)| n == name).count() > 1 {
                    panic!("Duplicate binding: {}", name);
                }
                
                // Compile the binding expression
                instrs.push(compile_expr(expr, &new_env, current_offset));
                
                // Store result on stack
                instrs.push(format!("mov [rsp - {}], rax", current_offset));
                
                // Add to environment
                new_env.insert(name.clone(), current_offset);
                current_offset += 8;
            }
            
            // Compile body with extended environment
            instrs.push(compile_expr(body, &new_env, current_offset));
            
            instrs.join("\n  ")
        }
        
        Expr::UnOp(op, expr) => {
            let expr_instrs = compile_expr(expr, env, stack_offset);
            let op_instr = match op {
                UnOp::Add1 => "add rax, 1",
                UnOp::Sub1 => "sub rax, 1",
                UnOp::Negate => "imul rax, -1",
            };
            format!("{}\n  {}", expr_instrs, op_instr)
        }
        
        Expr::BinOp(op, e1, e2) => {
            let mut instrs = Vec::new();
            
            // Evaluate left operand
            instrs.push(compile_expr(e1, env, stack_offset));
            
            // Save left operand on stack
            instrs.push(format!("mov [rsp - {}], rax", stack_offset));
            
            // Evaluate right operand
            instrs.push(compile_expr(e2, env, stack_offset + 8));
            
            // Perform operation
            match op {
                BinOp::Plus => {
                    instrs.push(format!("add rax, [rsp - {}]", stack_offset));
                }
                BinOp::Minus => {
                    instrs.push(format!("mov rbx, [rsp - {}]", stack_offset));
                    instrs.push("sub rbx, rax".to_string());
                    instrs.push("mov rax, rbx".to_string());
                }
                BinOp::Times => {
                    instrs.push(format!("imul rax, [rsp - {}]", stack_offset));
                }
            }
            
            instrs.join("\n  ")
        }
    }
}
```

### Task 4: Update Main Function

```rust
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input.snek> <output.s>", args[0]);
        std::process::exit(1);
    }

    let in_name = &args[1];
    let out_name = &args[2];

    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    let sexp = parse(&in_contents).unwrap();
    let expr = parse_expr(&sexp);
    
    // Start with empty environment and offset 8
    let env = HashMap::new();
    let instrs = compile_expr(&expr, &env, 8);
    
    let asm_program = format!(
        "section .text
global our_code_starts_here
our_code_starts_here:
  {}
  ret
",
        instrs
    );

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}
```

## Testing

Create comprehensive tests covering:
- Simple variables
- Nested let expressions
- Multiple bindings
- Binary operations
- Shadowing (same variable name in nested scopes)

**test/let_simple.snek:**
```scheme
(let ((x 5)) x)
```
Expected: `5`

**test/let_add.snek:**
```scheme
(let ((x 5) (y 10)) (+ x y))
```
Expected: `15`

**test/let_nested.snek:**
```scheme
(let ((x 5))
  (let ((y (+ x 1)))
    (+ x y)))
```
Expected: `11`

**test/shadowing.snek:**
```scheme
(let ((x 5))
  (let ((x 10))
    x))
```
Expected: `10`

## Error Cases

Your compiler should gracefully handle:
- Unbound variables: `(+ x 1)` with no binding for `x`
- Duplicate bindings: `(let ((x 1) (x 2)) x)`
- Use of keywords as identifiers: `(let ((let 5)) let)`

## Grading Rubric

- **Parser (20%)**: Correctly parses all valid Boa programs
- **Variable Lookup (20%)**: Correct environment management
- **Stack Management (25%)**: Proper stack offset calculation
- **Binary Operations (20%)**: Correct code for all operators
- **Testing (15%)**: Comprehensive test suite

## Extension Challenges (Optional)

1. **More Operators**: Add `/`, `%`, comparison operators
2. **Error Messages**: Better error reporting with context
3. **Dead Code Elimination**: Don't store unused bindings
4. **Register Allocation**: Use more registers before spilling to stack

## Common Pitfalls

1. **Wrong Stack Offsets**: Remember stack grows downward
2. **Operator Order**: For `(- a b)`, compute `b - a` in wrong order
3. **Environment Shadowing**: Make sure inner bindings shadow outer ones
4. **Temporary Storage**: Binary ops need temp storage for left operand

## Resources

- [x86-64 Stack Management](https://web.stanford.edu/class/cs107/guide/x86-64.html#stack)
- [Rust HashMap Documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html)

## Deliverables

Submit:
1. Complete source code
2. At least 15 test cases
3. README explaining your design decisions
4. Examples showing your compiler working

Good luck! This assignment significantly expands your compiler's capabilities.
