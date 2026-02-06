# Week 4: Diamondback - Functions and Calling Conventions

## Overview
Extend your compiler to support **Diamondback**, adding function definitions and function calls. This introduces stack frames, calling conventions, and proper function compilation with arguments and local variables.

## Learning Objectives
- Implement function definitions and calls
- Understand x86-64 calling conventions
- Manage stack frames with base pointers
- Handle argument passing
- Learn about proper tail call optimization (extension)

## The Diamondback Language

### Concrete Syntax
```
<prog> := <defn>* <expr>

<defn> := (fun (<name> <name>*) <expr>)

<expr> :=
  | <number> | true | false | input
  | <identifier>
  | (let ((<identifier> <expr>)+) <expr>)
  | (<op1> <expr>)
  | (<op2> <expr> <expr>)
  | (if <expr> <expr> <expr>)
  | (block <expr>+)
  | (loop <expr>)
  | (break <expr>)
  | (set! <identifier> <expr>)
  | (<name> <expr>*)                ; function call

<op1> := add1 | sub1 | negate | isnum | isbool | print
<op2> := + | - | * | < | > | <= | >= | =
```

### Stack Frame Layout

Each function has a stack frame:

```
        Higher addresses
        ---------------
rbp+24  | argument 2  |  (if 3+ args)
        ---------------
rbp+16  | argument 1  |  (if 2+ args)
        ---------------
rbp+8   | return addr |
        ---------------
rbp     | saved rbp   |  <- rbp points here
        ---------------
rbp-8   | local var 1 |
        ---------------
rbp-16  | local var 2 |
        ---------------
        Lower addresses
```

### Calling Convention

**Caller**:
1. Push arguments onto stack (right-to-left)
2. Call function
3. Clean up stack after return

**Callee**:
1. Save rbp
2. Set rbp = rsp
3. Allocate space for locals
4. Execute function body
5. Restore rbp
6. Return

### Print Function

Add a built-in `print` function:
- Takes one argument
- Prints the value (untagged)
- Returns the value

Implement in runtime:
```rust
#[no_mangle]
extern "C" fn snek_print(val: i64) -> i64 {
    if val & 1 == 0 {
        println!("{}", val >> 1);  // Number
    } else if val == 3 {
        println!("true");
    } else if val == 1 {
        println!("false");
    }
    val
}
```

### Examples

**Example 1 - Simple Function:**
```scheme
(fun (double x) (+ x x))

(double 5)
```
Result: `10`

**Example 2 - Recursion:**
```scheme
(fun (factorial n)
  (if (= n 1)
    1
    (* n (factorial (- n 1)))))

(factorial 5)
```
Result: `120`

**Example 3 - Multiple Arguments:**
```scheme
(fun (add3 x y z)
  (+ (+ x y) z))

(add3 1 2 3)
```
Result: `6`

**Example 4 - Local Variables:**
```scheme
(fun (compute x)
  (let ((y (* x 2))
        (z (+ y 1)))
    (- z x)))

(compute 10)
```
Result: `11`

## Implementation Strategy

### New AST Types

```rust
struct Program {
    defns: Vec<Definition>,
    main: Expr,
}

struct Definition {
    name: String,
    params: Vec<String>,
    body: Expr,
}

// Add to Expr enum:
enum Expr {
    // ... previous variants ...
    Call(String, Vec<Expr>),
}
```

### Function Compilation

Each function becomes a labeled section:

```asm
fun_double:
  push rbp
  mov rbp, rsp
  ; function body
  pop rbp
  ret
```

### Argument Access

Arguments are accessed via rbp:
- 1st arg: `[rbp + 16]`
- 2nd arg: `[rbp + 24]`  
- 3rd arg: `[rbp + 32]`
- etc.

### Function Calls

```asm
; For (add3 1 2 3)
mov rax, 6          ; 3rd arg (tagged)
push rax
mov rax, 4          ; 2nd arg (tagged)
push rax
mov rax, 2          ; 1st arg (tagged)
push rax
call fun_add3
add rsp, 24         ; Clean up (3 args * 8 bytes)
```

## Implementation Tasks

### Task 1: Parse Program Structure

```rust
fn parse_program(s: &Sexp) -> Program {
    match s {
        Sexp::List(items) => {
            let mut defns = vec![];
            let mut main_expr = None;
            
            for item in items {
                if let Some(defn) = try_parse_defn(item) {
                    defns.push(defn);
                } else if main_expr.is_none() {
                    main_expr = Some(parse_expr(item));
                } else {
                    panic!("Multiple main expressions");
                }
            }
            
            Program {
                defns,
                main: main_expr.expect("No main expression"),
            }
        }
        _ => panic!("Invalid program"),
    }
}

fn try_parse_defn(s: &Sexp) -> Option<Definition> {
    match s {
        Sexp::List(vec) => match &vec[..] {
            [Sexp::Atom(S(fun)), Sexp::List(signature), body] if fun == "fun" => {
                match &signature[..] {
                    [Sexp::Atom(S(name)), params @ ..] => {
                        let param_names: Vec<String> = params.iter().map(|p| {
                            match p {
                                Sexp::Atom(S(name)) => name.clone(),
                                _ => panic!("Invalid parameter"),
                            }
                        }).collect();
                        
                        Some(Definition {
                            name: name.clone(),
                            params: param_names,
                            body: parse_expr(body),
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}
```

### Task 2: Compile Function Definitions

```rust
fn compile_defn(defn: &Definition, label_counter: &mut i32) -> String {
    let mut instrs = vec![];
    
    // Function label
    instrs.push(format!("fun_{}:", defn.name));
    
    // Prologue: save rbp
    instrs.push("push rbp".to_string());
    instrs.push("mov rbp, rsp".to_string());
    
    // Build environment for parameters
    let mut env = HashMap::new();
    for (i, param) in defn.params.iter().enumerate() {
        // Parameters at rbp+16, rbp+24, rbp+32, ...
        env.insert(param.clone(), -(16 + i as i32 * 8));
    }
    
    // Compile body
    let body_instrs = compile_expr(&defn.body, &env, 8, label_counter, &None);
    instrs.push(body_instrs);
    
    // Epilogue: restore rbp and return
    instrs.push("pop rbp".to_string());
    instrs.push("ret".to_string());
    
    instrs.join("\n  ")
}
```

### Task 3: Compile Function Calls

```rust
Expr::Call(name, args) => {
    let mut instrs = vec![];
    
    // Push arguments right-to-left
    for arg in args.iter().rev() {
        instrs.push(compile_expr(arg, env, stack_offset, label_counter, break_target));
        instrs.push("push rax".to_string());
    }
    
    // Call function
    instrs.push(format!("call fun_{}", name));
    
    // Clean up stack
    if !args.is_empty() {
        instrs.push(format!("add rsp, {}", args.len() * 8));
    }
    
    instrs.join("\n  ")
}
```

### Task 4: Update Variable Access

Variables can now be parameters (positive offsets from rbp) or locals (negative offsets from rsp):

```rust
Expr::Var(name) => {
    match env.get(name) {
        Some(offset) => {
            if *offset < 0 {
                // Parameter: access via rbp
                format!("mov rax, [rbp - {}]", -offset)
            } else {
                // Local: access via rsp
                format!("mov rax, [rsp - {}]", offset)
            }
        }
        None => panic!("Unbound variable: {}", name),
    }
}
```

### Task 5: Compile Program

```rust
fn compile_program(prog: &Program) -> String {
    let mut label_counter = 0;
    let mut asm = vec![];
    
    // Compile all function definitions
    for defn in &prog.defns {
        asm.push(compile_defn(defn, &mut label_counter));
    }
    
    // Compile main entry point
    asm.push("our_code_starts_here:".to_string());
    asm.push("push rbp".to_string());
    asm.push("mov rbp, rsp".to_string());
    
    let main_instrs = compile_expr(&prog.main, &HashMap::new(), 8, &mut label_counter, &None);
    asm.push(main_instrs);
    
    asm.push("pop rbp".to_string());
    asm.push("ret".to_string());
    
    format!("section .text\nglobal our_code_starts_here\n{}", asm.join("\n"))
}
```

## Testing Requirements

Create at least 25 tests covering:
- Simple function calls
- Multiple arguments (0, 1, 2, 5+ arguments)
- Recursive functions
- Mutual recursion (two functions calling each other)
- Functions calling other functions
- Local variables in functions
- Mixed parameter and local variable access

## Error Handling

- Wrong number of arguments
- Calling undefined functions
- Shadowing parameters with let bindings

## Grading Rubric

- **Function Definitions (25%)**: Correct compilation
- **Function Calls (25%)**: Proper calling convention
- **Stack Frames (20%)**: Correct rbp/rsp management
- **Recursion (15%)**: Recursive calls work correctly
- **Testing (15%)**: Comprehensive test suite

## Extension Challenges

1. **Proper Tail Calls**: Optimize tail recursion
2. **First-Class Functions**: Functions as values
3. **Closures**: Capture environment in functions
4. **Variadic Functions**: Functions with variable argument counts

## Common Pitfalls

1. **Stack Alignment**: x86-64 requires 16-byte alignment
2. **Argument Order**: Remember right-to-left push order
3. **Parameter Offsets**: Positive from rbp, not rsp
4. **Cleanup**: Must pop arguments after call

## Deliverables

Submit:
1. Complete compiler with functions
2. At least 25 test cases
3. README explaining calling convention
4. Examples of interesting programs (factorial, fibonacci, etc.)

This assignment completes a fully functional compiler!
