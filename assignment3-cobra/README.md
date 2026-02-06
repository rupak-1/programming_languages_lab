# Week 3: Cobra - Booleans, Conditionals, and Loops

## Overview
Extend your compiler to support **Cobra**, adding booleans, conditionals (`if`), loops, mutation (`set!`), and runtime type checking. This introduces tagged values, control flow, and dynamic type errors.

## Learning Objectives
- Implement tagged value representation
- Generate conditional jumps
- Handle loops and break statements
- Add runtime error checking
- Understand value tagging for dynamic types

## The Cobra Language

### Concrete Syntax
```
<expr> :=
  | <number>
  | true | false
  | input
  | <identifier>
  | (let ((<identifier> <expr>)+) <expr>)
  | (add1 <expr>) | (sub1 <expr>) | (negate <expr>)
  | (+ <expr> <expr>) | (- <expr> <expr>) | (* <expr> <expr>)
  | (< <expr> <expr>) | (> <expr> <expr>) 
  | (<= <expr> <expr>) | (>= <expr> <expr>) | (= <expr> <expr>)
  | (isnum <expr>) | (isbool <expr>)
  | (if <expr> <expr> <expr>)
  | (block <expr>+)
  | (loop <expr>)
  | (break <expr>)
  | (set! <identifier> <expr>)
```

### Tagged Value Representation
Use the least significant bit to distinguish types:
- **Numbers**: Shift left by 1 (LSB = 0)
  - Value `5` → `0b1010` (10 in decimal)
- **Booleans**: Use specific patterns (LSB = 1)
  - `true` → `0x3` (0b11)
  - `false` → `0x1` (0b01)

### Semantics

**Booleans**: `true` and `false` are literals

**input**: Special variable containing command-line argument

**Comparisons**: Return boolean; error if operands not both numbers

**Equality**: Return boolean; error if operands have different types

**Type Checks**: `isnum` and `isbool` return booleans

**If**: Evaluate condition; if false (0b01), take else branch, otherwise take then branch

**Block**: Evaluate expressions in order, return last value

**Loop**: Repeatedly evaluate body forever (use `break` to exit)

**Break**: Exit innermost loop with given value

**Set!**: Mutate existing variable binding

### Examples

**Example 1 - Booleans:**
```scheme
(if true 5 10)
```
Result: `5`

**Example 2 - Comparisons:**
```scheme
(< 3 5)
```
Result: `true`

**Example 3 - Loop:**
```scheme
(let ((x 0))
  (loop
    (if (= x 10)
      (break x)
      (set! x (+ x 1)))))
```
Result: `10`

**Example 4 - Type Error:**
```scheme
(+ true 5)
```
Runtime error: "invalid argument"

## Implementation Strategy

### Value Tagging

```rust
const NUM_TAG: i64 = 0;
const NUM_TAG_MASK: i64 = 1;
const BOOL_TAG: i64 = 1;
const BOOL_TAG_MASK: i64 = 1;
const TRUE_VAL: i64 = 3;
const FALSE_VAL: i64 = 1;

fn encode_num(n: i32) -> i64 {
    (n as i64) << 1
}

fn decode_num(tagged: i64) -> i32 {
    (tagged >> 1) as i32
}
```

### Control Flow Labels

Generate unique labels for each `if`, `loop`, and `break`:

```rust
let mut label_counter = 0;

fn new_label(label_counter: &mut i32, name: &str) -> String {
    *label_counter += 1;
    format!("{}_{}", name, label_counter)
}
```

### Runtime Error Checking

Create helper function in `runtime/start.rs`:

```rust
#[no_mangle]
extern "C" fn snek_error(errcode: i64) {
    if errcode == 1 {
        eprintln!("invalid argument");
    } else if errcode == 2 {
        eprintln!("overflow");
    }
    std::process::exit(1);
}
```

## Implementation Tasks

### Task 1: Update AST

```rust
enum Expr {
    Num(i32),
    Bool(bool),
    Input,
    Var(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),
    Loop(Box<Expr>),
    Break(Box<Expr>),
    Set(String, Box<Expr>),
}

enum UnOp {
    Add1, Sub1, Negate,
    IsNum, IsBool,
}

enum BinOp {
    Plus, Minus, Times,
    Less, Greater, LessEq, GreaterEq, Equal,
}
```

### Task 2: Code Generation for Booleans

```rust
Expr::Bool(b) => {
    if *b {
        "mov rax, 3".to_string()  // true
    } else {
        "mov rax, 1".to_string()  // false
    }
}
```

### Task 3: Code Generation for If

```rust
Expr::If(cond, then_expr, else_expr) => {
    let else_label = new_label(label_counter, "if_else");
    let end_label = new_label(label_counter, "if_end");
    
    let mut instrs = vec![];
    
    // Evaluate condition
    instrs.push(compile_expr(cond, env, stack_offset, label_counter, break_target));
    
    // Check if false (0b01)
    instrs.push("cmp rax, 1".to_string());
    instrs.push(format!("je {}", else_label));
    
    // Then branch
    instrs.push(compile_expr(then_expr, env, stack_offset, label_counter, break_target));
    instrs.push(format!("jmp {}", end_label));
    
    // Else branch
    instrs.push(format!("{}:", else_label));
    instrs.push(compile_expr(else_expr, env, stack_offset, label_counter, break_target));
    
    // End
    instrs.push(format!("{}:", end_label));
    
    instrs.join("\n  ")
}
```

### Task 4: Type Checking for Binary Operations

```rust
// Before addition, check both operands are numbers
instrs.push("mov rbx, rax".to_string());
instrs.push("and rbx, 1".to_string());  // Check tag bit
instrs.push("cmp rbx, 0".to_string());
instrs.push("jne error".to_string());   // Jump to error if not number
```

### Task 5: Loop and Break

```rust
Expr::Loop(body) => {
    let loop_start = new_label(label_counter, "loop_start");
    let loop_end = new_label(label_counter, "loop_end");
    
    let mut instrs = vec![];
    instrs.push(format!("{}:", loop_start));
    instrs.push(compile_expr(body, env, stack_offset, label_counter, &Some(loop_end.clone())));
    instrs.push(format!("jmp {}", loop_start));
    instrs.push(format!("{}:", loop_end));
    
    instrs.join("\n  ")
}

Expr::Break(expr) => {
    match break_target {
        Some(label) => {
            let mut instrs = vec![];
            instrs.push(compile_expr(expr, env, stack_offset, label_counter, break_target));
            instrs.push(format!("jmp {}", label));
            instrs.join("\n  ")
        }
        None => panic!("break outside of loop"),
    }
}
```

## Testing Requirements

Create at least 20 tests covering:
- Boolean operations
- All comparison operators
- Nested if expressions
- Loops with break
- Type errors
- Set! with various patterns
- Mixed numeric and boolean operations

## Error Handling

Implement runtime errors for:
1. Type mismatches in operations
2. Overflow in arithmetic
3. Invalid equality comparisons

## Grading Rubric

- **Tagged Values (25%)**: Correct encoding/decoding
- **Control Flow (25%)**: If, loop, break work correctly
- **Type Checking (25%)**: Proper runtime errors
- **Mutation (15%)**: Set! works correctly
- **Testing (10%)**: Comprehensive test suite

## Extension Challenges

1. **Print Statement**: Add a print function for debugging
2. **Overflow Detection**: Check for integer overflow
3. **Better Error Messages**: Include line numbers
4. **Input Parsing**: Support multiple inputs

## Common Pitfalls

1. **Tag Confusion**: Remember to tag/untag values properly
2. **Comparison Order**: Ensure correct operand order
3. **Break Outside Loop**: Check for valid break context
4. **Mutation Scope**: Set! only works on existing bindings

## Deliverables

Submit:
1. Complete compiler with all features
2. At least 20 test cases including error cases
3. Documentation of tagging scheme
4. Examples demonstrating all features
