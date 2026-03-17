// Assignment 2: Boa Compiler - Starter Code
// TODO: Complete this compiler implementation
//
// Your task is to implement a compiler for the Boa language
// that compiles expressions with let bindings to x86-64 assembly.
//
// Boa extends Adder with:
// - Variables (identifiers)
// - Let expressions with multiple bindings
// - Binary operations: +, -, *

use im::HashMap;
use sexp::Atom::*;
use sexp::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;

// ============= Abstract Syntax Tree =============

/// Unary operators
#[derive(Debug)]
enum Op1 {
    Add1,
    Sub1,
}

/// Binary operators
#[derive(Debug)]
enum Op2 {
    Plus,
    Minus,
    Times,
}

/// The Boa expression AST
///
/// Grammar:
/// <expr> := <number>
///         | <identifier>
///         | (let ((<identifier> <expr>)+) <expr>)
///         | (add1 <expr>) | (sub1 <expr>)
///         | (+ <expr> <expr>) | (- <expr> <expr>) | (* <expr> <expr>)
/// <identifier> := [a-zA-Z][a-zA-Z0-9_-]*  (but not reserved words)
#[derive(Debug)]
enum Expr {
    Number(i32),
    Id(String),
    Let(Vec<(String, Expr)>, Box<Expr>),
    UnOp(Op1, Box<Expr>),
    BinOp(Op2, Box<Expr>, Box<Expr>),
}

// ============= Assembly Representation =============

/// Values that can appear in assembly instructions
#[derive(Debug)]
enum Val {
    Reg(Reg),
    Imm(i32),
    RegOffset(Reg, i32), // e.g., [rsp - 8]
}

/// Registers we use
#[derive(Debug)]
enum Reg {
    RAX,
    RSP,
}

/// Assembly instructions we generate
#[derive(Debug)]
enum Instr {
    IMov(Val, Val),
    IAdd(Val, Val),
    ISub(Val, Val),
    IMul(Val, Val),
}

// ============= Parsing =============

/// Parse an S-expression into our Expr AST
///
/// Examples of valid Boa expressions:
/// 42 -> Number(42)
/// x -> Id("x")
/// (add1 5) -> UnOp(Add1, Number(5))
/// (+ 1 2) -> BinOp(Plus, Number(1), Number(2))
/// (let ((x 5)) x) -> Let([("x", Number(5))], Id("x"))
/// (let ((x 5) (y 6)) (+ x y)) -> Let([("x", Number(5)), ("y", Number(6))], BinOp(...))
///
/// Error handling:
/// - Invalid syntax: panic!("Invalid")
/// - Number out of i32 range: panic!("Invalid")
fn parse_expr(s: &Sexp) -> Expr {
    match s {
        // Integer literal: convert to i32, reject if out of range
        Sexp::Atom(I(n)) => Expr::Number(
            i32::try_from(*n).unwrap_or_else(|_| panic!("Invalid: number {} out of i32 range", n)),
        ),

        // Identifier: must not be a reserved keyword (let, add1, sub1)
        Sexp::Atom(S(id)) => {
            match id.as_str() {
                "let" | "add1" | "sub1" => {
                    panic!("Invalid: '{}' is a reserved keyword and cannot be used as identifier", id)
                }
                _ => Expr::Id(id.clone()),
            }
        }

        // List form: (op expr...) or (let ((id expr)...) body)
        Sexp::List(vec) => match &vec[..] {
            // Let: (let ((x 1) (y 2)) body) — parse each binding, check for duplicates
            [Sexp::Atom(S(kw)), Sexp::List(bindings), body] if kw == "let" => {
                if bindings.is_empty() {
                    panic!("Invalid: let must have at least one binding");
                }
                let mut scope_bindings = Vec::new();
                let mut seen_ids = std::collections::HashSet::new();
                for b in bindings {
                    let (id, expr) = parse_bind(b);
                    if !seen_ids.insert(id.clone()) {
                        panic!("Duplicate binding: variable '{}' appears twice in same let block", id);
                    }
                    scope_bindings.push((id, expr));
                }
                Expr::Let(scope_bindings, Box::new(parse_expr(body)))
            }
            // Unary ops: (add1 e) and (sub1 e)
            [Sexp::Atom(S(op)), e] if op == "add1" => {
                Expr::UnOp(Op1::Add1, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "sub1" => {
                Expr::UnOp(Op1::Sub1, Box::new(parse_expr(e)))
            }
            // Binary ops: (+ e1 e2), (- e1 e2), (* e1 e2)
            [Sexp::Atom(S(op)), e1, e2] if op == "+" => {
                Expr::BinOp(Op2::Plus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == "-" => {
                Expr::BinOp(Op2::Minus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == "*" => {
                Expr::BinOp(Op2::Times, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            _ => panic!("Invalid: expected add1, sub1, +, -, *, or let; got malformed list {:?}", vec),
        },

        _ => panic!("Invalid: expected number, identifier, or list expression; got {:?}", s),
    }
}

/// Parse a single binding from a let expression
///
/// A binding looks like: (x 5) or (my-var (+ 1 2))
/// Returns a tuple of (variable_name, expression)
///
/// Error handling:
/// - Invalid binding syntax: panic!("Invalid")
fn parse_bind(s: &Sexp) -> (String, Expr) {
    // Binding is (identifier expr), e.g. (x 5) or (y (+ 1 2))
    match s {
        Sexp::List(vec) => match &vec[..] {
            [Sexp::Atom(S(id)), e] => {
                match id.as_str() {
                    "let" | "add1" | "sub1" => {
                        panic!("Invalid binding: '{}' is a reserved keyword and cannot be used as variable name", id)
                    }
                    _ => (id.clone(), parse_expr(e)),
                }
            }
            _ => panic!("Invalid binding: expected (identifier expr), got {:?}", vec),
        },
        _ => panic!("Invalid binding: expected list of (identifier expr), got {:?}", s),
    }
}

// ============= Compilation =============

/// Compile an expression to a list of assembly instructions
///
/// Parameters:
/// - e: the expression to compile
/// - si: stack index - the next available stack slot (starts at 2)
/// Stack slots are at [rsp - 8*si], e.g., si=2 means [rsp - 16]
/// - env: environment mapping variable names to stack offsets
///
/// The compiled code should leave its result in RAX.
///
/// Stack layout:
/// [rsp - 8] : reserved (return address area)
/// [rsp - 16] : first variable (si=2)
/// [rsp - 24] : second variable (si=3)
/// ...
///
/// Examples:
/// Number(5) -> [IMov(Reg(RAX), Imm(5))]
///
/// UnOp(Add1, Number(5)) ->
/// [IMov(Reg(RAX), Imm(5)), IAdd(Reg(RAX), Imm(1))]
///
/// BinOp(Plus, Number(1), Number(2)) ->
/// 1. Compile left operand (result in RAX)
/// 2. Save RAX to stack at [rsp - 8*si]
/// 3. Compile right operand (result in RAX)
/// 4. Add stack value to RAX
///
/// Let([(x, 5)], Id(x)) ->
/// 1. Compile binding expression (5)
/// 2. Store result at stack slot
/// 3. Add x -> stack_offset to environment
/// 4. Compile body with updated environment
fn compile_to_instrs(e: &Expr, si: i32, env: &HashMap<String, i32>) -> Vec<Instr> {
    match e {
        // Number: load immediate into RAX (result register)
        Expr::Number(n) => vec![Instr::IMov(Val::Reg(Reg::RAX), Val::Imm(*n))],

        // Id: look up variable's stack offset in env, load value from [rsp + offset] into RAX
        Expr::Id(id) => {
            let stack_off = env
                .get(id)
                .copied()
                .unwrap_or_else(|| panic!("Unbound variable identifier {}: not found in current scope", id));
            vec![Instr::IMov(
                Val::Reg(Reg::RAX),
                Val::RegOffset(Reg::RSP, stack_off),
            )]
        }

        // UnOp: compile subexpr (result in RAX), then add/sub 1
        Expr::UnOp(Op1::Add1, sub) => {
            let mut code = compile_to_instrs(sub, si, env);
            code.push(Instr::IAdd(Val::Reg(Reg::RAX), Val::Imm(1)));
            code
        }
        Expr::UnOp(Op1::Sub1, sub) => {
            let mut code = compile_to_instrs(sub, si, env);
            code.push(Instr::ISub(Val::Reg(Reg::RAX), Val::Imm(1)));
            code
        }

        // BinOp: left and right both need RAX; save left to stack before compiling right
        Expr::BinOp(op, left, right) => {
            // 1. Compile left operand; result ends up in RAX
            let mut code = compile_to_instrs(left, si, env);
            let slot_off = -8 * si;
            // 2. Spill left result to [rsp - 8*si] so we can use RAX for right
            code.push(Instr::IMov(
                Val::RegOffset(Reg::RSP, slot_off),
                Val::Reg(Reg::RAX),
            ));
            // 3. Compile right operand; for Minus we need si+2 to reserve si+1 for temp (no RBX)
            let right_si = match op {
                Op2::Minus => si + 2,
                _ => si + 1,
            };
            code.append(&mut compile_to_instrs(right, right_si, env));
            let stack_slot = Val::RegOffset(Reg::RSP, slot_off);
            // 4. Combine: RAX has right, stack has left; produce left OP right in RAX
            match op {
                Op2::Plus => code.push(Instr::IAdd(Val::Reg(Reg::RAX), stack_slot)),
                Op2::Minus => {
                    // sub dst, src => dst = dst - src. We need left - right.
                    // Save right to si+1, load left to RAX, then sub right.
                    let right_off = -8 * (si + 1);
                    code.push(Instr::IMov(
                        Val::RegOffset(Reg::RSP, right_off),
                        Val::Reg(Reg::RAX),
                    ));
                    code.push(Instr::IMov(Val::Reg(Reg::RAX), stack_slot));
                    code.push(Instr::ISub(
                        Val::Reg(Reg::RAX),
                        Val::RegOffset(Reg::RSP, right_off),
                    ));
                }
                Op2::Times => code.push(Instr::IMul(Val::Reg(Reg::RAX), stack_slot)),
            }
            code
        }

        // Let: evaluate bindings left-to-right, extend env, compile body in new scope
        Expr::Let(bindings, body) => {
            let mut code = Vec::new();
            let mut scope_env = env.clone();
            let mut next_slot = si;
            for (id, expr) in bindings {
                let slot_off = -8 * next_slot;
                code.append(&mut compile_to_instrs(expr, next_slot, &scope_env));
                // Store binding value at this stack slot
                code.push(Instr::IMov(
                    Val::RegOffset(Reg::RSP, slot_off),
                    Val::Reg(Reg::RAX),
                ));
                scope_env = scope_env.update(id.clone(), slot_off);
                next_slot += 1;
            }
            code.append(&mut compile_to_instrs(body, next_slot, &scope_env));
            code
        }
    }
}

// ============= Code Generation =============

/// Convert a Val to its assembly string representation
fn val_to_str(v: &Val) -> String {
    match v {
        Val::Reg(Reg::RAX) => String::from("rax"),
        Val::Reg(Reg::RSP) => String::from("rsp"),
        Val::Imm(n) => format!("{}", n),
        Val::RegOffset(Reg::RSP, offset) => format!("[rsp + {}]", offset),
        Val::RegOffset(Reg::RAX, offset) => format!("[rax + {}]", offset),
    }
}

/// Convert an Instr to its assembly string representation
fn instr_to_str(i: &Instr) -> String {
    match i {
        Instr::IMov(dst, src) => format!("mov {}, {}", val_to_str(dst), val_to_str(src)),
        Instr::IAdd(dst, src) => format!("add {}, {}", val_to_str(dst), val_to_str(src)),
        Instr::ISub(dst, src) => format!("sub {}, {}", val_to_str(dst), val_to_str(src)),
        Instr::IMul(dst, src) => format!("imul {}, {}", val_to_str(dst), val_to_str(src)),
    }
}

/// Compile an expression to a complete assembly string
fn compile(e: &Expr) -> String {
    let env: HashMap<String, i32> = HashMap::new();
    let instrs = compile_to_instrs(e, 2, &env);
    instrs
        .iter()
        .map(|i| instr_to_str(i))
        .collect::<Vec<_>>()
        .join("\n  ")
}

// ============= Main =============

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input.snek> <output.s>", args[0]);
        std::process::exit(1);
    }

    let in_name = &args[1];
    let out_name = &args[2];

    // Read input file
    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    // Parse S-expression from text
    let sexp = parse(&in_contents).unwrap_or_else(|e| panic!("Invalid: failed to parse S-expression: {}", e));

    // Convert S-expression to our AST
    let expr = parse_expr(&sexp);

    // Generate assembly instructions
    let instrs = compile(&expr);

    // Wrap instructions in assembly program template
    let asm_program = format!(
        "section .text
global our_code_starts_here
our_code_starts_here:
  {}
  ret
",
        instrs
    );

    // Write output assembly file
    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}

// ============= TESTS =============
//
// Run with: cargo test
//
// These tests help verify your implementation. Uncomment and add more!

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to parse a string directly
    fn parse_str(s: &str) -> Expr {
        parse_expr(&parse(s).unwrap())
    }

    // ===== Parsing Tests =====

    #[test]
    fn test_parse_number() {
        let expr = parse_str("42");
        match expr {
            Expr::Number(42) => (),
            _ => panic!("Expected Number(42), got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_identifier() {
        let expr = parse_str("x");
        match expr {
            Expr::Id(name) => assert_eq!(name, "x"),
            _ => panic!("Expected Id(\"x\"), got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_add1() {
        let expr = parse_str("(add1 5)");
        match expr {
            Expr::UnOp(Op1::Add1, _) => (),
            _ => panic!("Expected UnOp(Add1, ...), got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_binary_plus() {
        let expr = parse_str("(+ 1 2)");
        match expr {
            Expr::BinOp(Op2::Plus, _, _) => (),
            _ => panic!("Expected BinOp(Plus, ...), got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_let_simple() {
        let expr = parse_str("(let ((x 5)) x)");
        match expr {
            Expr::Let(bindings, _) => {
                assert_eq!(bindings.len(), 1);
                assert_eq!(bindings[0].0, "x");
            }
            _ => panic!("Expected Let, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_let_multiple_bindings() {
        let expr = parse_str("(let ((x 5) (y 6)) (+ x y))");
        match expr {
            Expr::Let(bindings, _) => {
                assert_eq!(bindings.len(), 2);
            }
            _ => panic!("Expected Let with 2 bindings, got {:?}", expr),
        }
    }

    // ===== Error Tests =====

    #[test]
    #[should_panic(expected = "Duplicate binding")]
    fn test_duplicate_binding() {
        let expr = parse_str("(let ((x 1) (x 2)) x)");
        let env: HashMap<String, i32> = HashMap::new();
        compile_to_instrs(&expr, 2, &env);
    }

    #[test]
    #[should_panic(expected = "let must have at least one binding")]
    fn test_empty_let() {
        parse_str("(let () 5)");
    }

    #[test]
    #[should_panic(expected = "reserved keyword")]
    fn test_reserved_word_as_binding_name() {
        parse_str("(let ((let 5)) let)");
    }

    #[test]
    #[should_panic(expected = "Unbound variable identifier y")]
    fn test_unbound_variable() {
        let expr = parse_str("y");
        let env: HashMap<String, i32> = HashMap::new();
        compile_to_instrs(&expr, 2, &env);
    }

    // ===== Compilation Tests =====

    #[test]
    fn test_compile_number() {
        let expr = Expr::Number(42);
        let env: HashMap<String, i32> = HashMap::new();
        let instrs = compile_to_instrs(&expr, 2, &env);
        assert_eq!(instrs.len(), 1);
    }

    // Add more tests as you implement features!
}
