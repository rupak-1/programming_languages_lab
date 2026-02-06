// Week 2: Boa Compiler Solution
// src/main.rs

use sexp::*;
use sexp::Atom::*;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;

// Abstract Syntax Tree
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

/// Parse an S-expression into our Expr AST
fn parse_expr(s: &Sexp) -> Expr {
    match s {
        Sexp::Atom(I(n)) => Expr::Num(i32::try_from(*n).unwrap()),
        
        Sexp::Atom(S(name)) => {
            // Check for reserved keywords
            match name.as_str() {
                "let" | "add1" | "sub1" | "negate" => {
                    panic!("Invalid use of keyword as identifier: {}", name)
                }
                _ => Expr::Var(name.to_string()),
            }
        }
        
        Sexp::List(vec) => match &vec[..] {
            // Let expression: (let ((x 1) (y 2)) body)
            [Sexp::Atom(S(keyword)), Sexp::List(bindings), body] if keyword == "let" => {
                if bindings.is_empty() {
                    panic!("Let must have at least one binding");
                }
                
                let mut parsed_bindings = Vec::new();
                let mut seen_names = std::collections::HashSet::new();
                
                for binding in bindings {
                    match binding {
                        Sexp::List(pair) => match &pair[..] {
                            [Sexp::Atom(S(name)), expr] => {
                                // Check for duplicate bindings
                                if !seen_names.insert(name.clone()) {
                                    panic!("Duplicate binding: {}", name);
                                }
                                // Check for keyword as binding name
                                match name.as_str() {
                                    "let" | "add1" | "sub1" | "negate" => {
                                        panic!("Cannot use keyword as binding name: {}", name)
                                    }
                                    _ => {}
                                }
                                parsed_bindings.push((name.to_string(), parse_expr(expr)));
                            }
                            _ => panic!("Invalid binding: {:?}", pair),
                        },
                        _ => panic!("Invalid binding: {:?}", binding),
                    }
                }
                
                Expr::Let(parsed_bindings, Box::new(parse_expr(body)))
            }
            
            // Unary operations
            [Sexp::Atom(S(op)), e] if op == "add1" => {
                Expr::UnOp(UnOp::Add1, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "sub1" => {
                Expr::UnOp(UnOp::Sub1, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "negate" => {
                Expr::UnOp(UnOp::Negate, Box::new(parse_expr(e)))
            }
            
            // Binary operations
            [Sexp::Atom(S(op)), e1, e2] if op == "+" => {
                Expr::BinOp(BinOp::Plus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == "-" => {
                Expr::BinOp(BinOp::Minus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == "*" => {
                Expr::BinOp(BinOp::Times, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            
            _ => panic!("Invalid expression: {:?}", vec),
        },
        
        _ => panic!("Invalid expression: {:?}", s),
    }
}

/// Compile an expression to x86-64 assembly
/// env: maps variable names to stack offsets
/// stack_offset: current stack offset for next temp/variable
fn compile_expr(e: &Expr, env: &HashMap<String, i32>, stack_offset: i32) -> String {
    match e {
        Expr::Num(n) => format!("mov rax, {}", n),
        
        Expr::Var(name) => {
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
                // Compile binding expression with current environment
                instrs.push(compile_expr(expr, &new_env, current_offset));
                
                // Store result on stack
                instrs.push(format!("mov [rsp - {}], rax", current_offset));
                
                // Add to environment for subsequent bindings and body
                new_env.insert(name.clone(), current_offset);
                current_offset += 8;
            }
            
            // Compile body with full environment
            instrs.push(compile_expr(body, &new_env, current_offset));
            
            instrs.join("\n  ")
        }
        
        Expr::UnOp(op, expr) => {
            let mut instrs = Vec::new();
            instrs.push(compile_expr(expr, env, stack_offset));
            
            match op {
                UnOp::Add1 => instrs.push("add rax, 1".to_string()),
                UnOp::Sub1 => instrs.push("sub rax, 1".to_string()),
                UnOp::Negate => instrs.push("imul rax, -1".to_string()),
            }
            
            instrs.join("\n  ")
        }
        
        Expr::BinOp(op, e1, e2) => {
            let mut instrs = Vec::new();
            
            // Evaluate left operand
            instrs.push(compile_expr(e1, env, stack_offset));
            
            // Save left operand on stack
            instrs.push(format!("mov [rsp - {}], rax", stack_offset));
            
            // Evaluate right operand (with incremented stack offset)
            instrs.push(compile_expr(e2, env, stack_offset + 8));
            
            // Perform operation
            match op {
                BinOp::Plus => {
                    // rax = right, [rsp - stack_offset] = left
                    // result = left + right
                    instrs.push(format!("add rax, [rsp - {}]", stack_offset));
                }
                BinOp::Minus => {
                    // rax = right, [rsp - stack_offset] = left
                    // result = left - right
                    instrs.push(format!("mov rbx, [rsp - {}]", stack_offset));
                    instrs.push("sub rbx, rax".to_string());
                    instrs.push("mov rax, rbx".to_string());
                }
                BinOp::Times => {
                    // rax = right, [rsp - stack_offset] = left
                    // result = left * right
                    instrs.push(format!("imul rax, [rsp - {}]", stack_offset));
                }
            }
            
            instrs.join("\n  ")
        }
    }
}

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

    let sexp = parse(&in_contents).unwrap_or_else(|e| {
        panic!("Parse error: {}", e)
    });
    let expr = parse_expr(&sexp);
    
    // Start with empty environment and stack offset 8
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_let() {
        let sexp = parse("(let ((x 5)) x)").unwrap();
        let expr = parse_expr(&sexp);
        assert!(matches!(expr, Expr::Let(_, _)));
    }

    #[test]
    fn test_compile_variable() {
        let expr = Expr::Let(
            vec![("x".to_string(), Expr::Num(5))],
            Box::new(Expr::Var("x".to_string())),
        );
        let env = HashMap::new();
        let asm = compile_expr(&expr, &env, 8);
        assert!(asm.contains("mov rax, 5"));
        assert!(asm.contains("[rsp - 8]"));
    }

    #[test]
    fn test_compile_binop() {
        let expr = Expr::BinOp(
            BinOp::Plus,
            Box::new(Expr::Num(3)),
            Box::new(Expr::Num(4)),
        );
        let env = HashMap::new();
        let asm = compile_expr(&expr, &env, 8);
        assert!(asm.contains("add rax"));
    }

    #[test]
    #[should_panic(expected = "Unbound variable")]
    fn test_unbound_variable() {
        let expr = Expr::Var("x".to_string());
        let env = HashMap::new();
        compile_expr(&expr, &env, 8);
    }

    #[test]
    #[should_panic(expected = "Duplicate binding")]
    fn test_duplicate_binding() {
        let sexp = parse("(let ((x 1) (x 2)) x)").unwrap();
        parse_expr(&sexp);
    }
}
