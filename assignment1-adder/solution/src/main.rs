// Week 1: Adder Compiler Solution
// src/main.rs

use sexp::*;
use sexp::Atom::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;

// Abstract Syntax Tree definition
#[derive(Debug)]
enum Expr {
    Num(i32),
    Add1(Box<Expr>),
    Sub1(Box<Expr>),
    Negate(Box<Expr>),
}

/// Parse an S-expression into our Expr AST
fn parse_expr(s: &Sexp) -> Expr {
    match s {
        // Base case: number literal
        Sexp::Atom(I(n)) => {
            // Try to convert to i32, panic if out of range
            Expr::Num(i32::try_from(*n).unwrap_or_else(|_| {
                panic!("Number {} out of range for 32-bit integer", n)
            }))
        }
        
        // Recursive cases: operations
        Sexp::List(vec) => match &vec[..] {
            [Sexp::Atom(S(op)), e] if op == "add1" => {
                Expr::Add1(Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "sub1" => {
                Expr::Sub1(Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "negate" => {
                Expr::Negate(Box::new(parse_expr(e)))
            }
            _ => panic!("Invalid expression: {:?}", vec),
        },
        
        _ => panic!("Invalid expression: {:?}", s),
    }
}

/// Compile an Expr to x86-64 assembly instructions
/// The result is left in the rax register
fn compile_expr(e: &Expr) -> String {
    match e {
        // For a number, move it directly into rax
        Expr::Num(n) => format!("mov rax, {}", n),
        
        // For add1: compile subexpression, then add 1 to rax
        Expr::Add1(subexpr) => {
            let subexpr_instrs = compile_expr(subexpr);
            format!("{}\n  add rax, 1", subexpr_instrs)
        }
        
        // For sub1: compile subexpression, then subtract 1 from rax
        Expr::Sub1(subexpr) => {
            let subexpr_instrs = compile_expr(subexpr);
            format!("{}\n  sub rax, 1", subexpr_instrs)
        }
        
        // For negate: compile subexpression, then negate rax
        // We use imul to multiply by -1
        Expr::Negate(subexpr) => {
            let subexpr_instrs = compile_expr(subexpr);
            format!("{}\n  imul rax, -1", subexpr_instrs)
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Check for correct number of arguments
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

    // Parse S-expression
    let sexp = parse(&in_contents)
        .unwrap_or_else(|e| panic!("Parse error: {}", e));
    
    // Convert to our AST
    let expr = parse_expr(&sexp);
    
    // Generate assembly
    let instrs = compile_expr(&expr);
    
    // Wrap in assembly template
    let asm_program = format!(
        "section .text
global our_code_starts_here
our_code_starts_here:
  {}
  ret
",
        instrs
    );

    // Write output file
    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm_program.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let sexp = parse("42").unwrap();
        let expr = parse_expr(&sexp);
        assert!(matches!(expr, Expr::Num(42)));
    }

    #[test]
    fn test_parse_add1() {
        let sexp = parse("(add1 5)").unwrap();
        let expr = parse_expr(&sexp);
        assert!(matches!(expr, Expr::Add1(_)));
    }

    #[test]
    fn test_compile_number() {
        let expr = Expr::Num(42);
        let asm = compile_expr(&expr);
        assert_eq!(asm, "mov rax, 42");
    }

    #[test]
    fn test_compile_add1() {
        let expr = Expr::Add1(Box::new(Expr::Num(5)));
        let asm = compile_expr(&expr);
        assert!(asm.contains("mov rax, 5"));
        assert!(asm.contains("add rax, 1"));
    }

    #[test]
    fn test_compile_nested() {
        let expr = Expr::Add1(Box::new(
            Expr::Sub1(Box::new(Expr::Num(10)))
        ));
        let asm = compile_expr(&expr);
        assert!(asm.contains("mov rax, 10"));
        assert!(asm.contains("sub rax, 1"));
        assert!(asm.contains("add rax, 1"));
    }
}
