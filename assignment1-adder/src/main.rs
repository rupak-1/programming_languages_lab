// Adder Language Compiler Implementation
// Compiles arithmetic expressions to x86-64 assembly code

use sexp::*;
use sexp::Atom::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;

// Abstract Syntax Tree representation
#[derive(Debug)]
enum Expr {
    Num(i32),
    Add1(Box<Expr>),
    Sub1(Box<Expr>),
    Negate(Box<Expr>),
}

/// Converts an S-expression into our AST representation
fn parse_expr(s: &Sexp) -> Expr {
    match s {
        // Handle integer literals
        Sexp::Atom(I(n)) => {
            // Convert to i32, handling overflow
            Expr::Num(i32::try_from(*n).unwrap_or_else(|_| {
                panic!("Integer value {} exceeds i32 range", n)
            }))
        }
        
        // Handle operation expressions
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
            _ => panic!("Malformed expression structure: {:?}", vec),
        },
        
        _ => panic!("Invalid expression: {:?}", s),
    }
}

/// Generates x86-64 assembly code from an AST expression
/// The final result will be stored in the rax register
fn compile_expr(e: &Expr) -> String {
    match e {
        // Load constant value into rax
        Expr::Num(n) => format!("mov rax, {}", n),
        
        // Increment operation: evaluate subexpression, then add 1
        Expr::Add1(subexpr) => {
            let subexpr_code = compile_expr(subexpr);
            format!("{}\n  add rax, 1", subexpr_code)
        }
        
        // Decrement operation: evaluate subexpression, then subtract 1
        Expr::Sub1(subexpr) => {
            let subexpr_code = compile_expr(subexpr);
            format!("{}\n  sub rax, 1", subexpr_code)
        }
        
        // Negation operation: evaluate subexpression, then multiply by -1
        Expr::Negate(subexpr) => {
            let subexpr_code = compile_expr(subexpr);
            format!("{}\n  imul rax, -1", subexpr_code)
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Error: Expected 2 arguments\nUsage: {} <input.snek> <output.s>", args[0]);
        std::process::exit(1);
    }

    let in_name = &args[1];
    let out_name = &args[2];

    // Read input file
    let mut in_file = File::open(in_name)?;
    let mut in_contents = String::new();
    in_file.read_to_string(&mut in_contents)?;

    // Parse S-expression from text
    let sexp = parse(&in_contents).unwrap_or_else(|e| {
        panic!("Parse error: {}", e)
    });
    
    // Convert S-expression to our AST
    let expr = parse_expr(&sexp);
    
    // Generate assembly instructions
    let instrs = compile_expr(&expr);
    
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer_literal() {
        let sexp = parse("73").unwrap();
        let expr = parse_expr(&sexp);
        assert!(matches!(expr, Expr::Num(73)));
    }

    #[test]
    fn test_parse_add1_operation() {
        let sexp = parse("(add1 8)").unwrap();
        let expr = parse_expr(&sexp);
        assert!(matches!(expr, Expr::Add1(_)));
    }

    #[test]
    fn test_parse_sub1_operation() {
        let sexp = parse("(sub1 15)").unwrap();
        let expr = parse_expr(&sexp);
        assert!(matches!(expr, Expr::Sub1(_)));
    }

    #[test]
    fn test_parse_negate_operation() {
        let sexp = parse("(negate 9)").unwrap();
        let expr = parse_expr(&sexp);
        assert!(matches!(expr, Expr::Negate(_)));
    }

    #[test]
    fn test_compile_integer() {
        let expr = Expr::Num(73);
        let asm = compile_expr(&expr);
        assert_eq!(asm, "mov rax, 73");
    }

    #[test]
    fn test_compile_add1() {
        let expr = Expr::Add1(Box::new(Expr::Num(8)));
        let asm = compile_expr(&expr);
        assert!(asm.contains("mov rax, 8"));
        assert!(asm.contains("add rax, 1"));
    }

    #[test]
    fn test_compile_sub1() {
        let expr = Expr::Sub1(Box::new(Expr::Num(20)));
        let asm = compile_expr(&expr);
        assert!(asm.contains("mov rax, 20"));
        assert!(asm.contains("sub rax, 1"));
    }

    #[test]
    fn test_compile_negate() {
        let expr = Expr::Negate(Box::new(Expr::Num(12)));
        let asm = compile_expr(&expr);
        assert!(asm.contains("mov rax, 12"));
        assert!(asm.contains("imul rax, -1"));
    }

    #[test]
    fn test_compile_chained_operations() {
        let expr = Expr::Sub1(Box::new(
            Expr::Add1(Box::new(Expr::Num(25)))
        ));
        let asm = compile_expr(&expr);
        assert!(asm.contains("mov rax, 25"));
        assert!(asm.contains("add rax, 1"));
        assert!(asm.contains("sub rax, 1"));
    }

    #[test]
    fn test_compile_nested_negate() {
        let expr = Expr::Negate(Box::new(
            Expr::Add1(Box::new(Expr::Num(6)))
        ));
        let asm = compile_expr(&expr);
        assert!(asm.contains("mov rax, 6"));
        assert!(asm.contains("add rax, 1"));
        assert!(asm.contains("imul rax, -1"));
    }
}
