// Cobra compiler: tagged values, control flow, runtime checks

use sexp::Atom::*;
use sexp::*;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
enum UnOp {
    Add1,
    Sub1,
    Negate,
    IsNum,
    IsBool,
}

#[derive(Debug, Clone)]
enum BinOp {
    Plus,
    Minus,
    Times,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    Equal,
}

fn reserved_word(sym: &str) -> bool {
    matches!(
        sym,
        "let" | "add1"
            | "sub1"
            | "negate"
            | "+"
            | "-"
            | "*"
            | "<"
            | ">"
            | "<="
            | ">="
            | "="
            | "isnum"
            | "isbool"
            | "if"
            | "block"
            | "loop"
            | "break"
            | "set!"
            | "true"
            | "false"
            | "input"
    )
}

fn parse_expr(s: &Sexp) -> Expr {
    match s {
        Sexp::Atom(I(n)) => Expr::Num(i32::try_from(*n).unwrap()),

        Sexp::Atom(S(name)) => match name.as_str() {
            "true" => Expr::Bool(true),
            "false" => Expr::Bool(false),
            "input" => Expr::Input,
            _ => {
                if reserved_word(name) {
                    panic!("Invalid use of keyword as identifier: {}", name);
                }
                Expr::Var(name.to_string())
            }
        },

        Sexp::List(vec) => match &vec[..] {
            [Sexp::Atom(S(kw)), Sexp::List(bindings), body] if kw == "let" => {
                if bindings.is_empty() {
                    panic!("Let must have at least one binding");
                }
                let mut pairs = Vec::new();
                let mut seen = HashSet::new();
                for b in bindings {
                    match b {
                        Sexp::List(pair) => match &pair[..] {
                            [Sexp::Atom(S(nm)), rhs] => {
                                if !seen.insert(nm.clone()) {
                                    panic!("Duplicate binding: {}", nm);
                                }
                                if reserved_word(nm) {
                                    panic!("Cannot use keyword as binding name: {}", nm);
                                }
                                pairs.push((nm.to_string(), parse_expr(rhs)));
                            }
                            _ => panic!("Invalid binding: {:?}", pair),
                        },
                        _ => panic!("Invalid binding: {:?}", b),
                    }
                }
                Expr::Let(pairs, Box::new(parse_expr(body)))
            }

            [Sexp::Atom(S(op)), e] if op == "add1" => {
                Expr::UnOp(UnOp::Add1, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "sub1" => {
                Expr::UnOp(UnOp::Sub1, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "negate" => {
                Expr::UnOp(UnOp::Negate, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "isnum" => {
                Expr::UnOp(UnOp::IsNum, Box::new(parse_expr(e)))
            }
            [Sexp::Atom(S(op)), e] if op == "isbool" => {
                Expr::UnOp(UnOp::IsBool, Box::new(parse_expr(e)))
            }

            [Sexp::Atom(S(op)), e1, e2] if op == "+" => {
                Expr::BinOp(BinOp::Plus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == "-" => {
                Expr::BinOp(BinOp::Minus, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == "*" => {
                Expr::BinOp(BinOp::Times, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == "<" => {
                Expr::BinOp(BinOp::Less, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == ">" => {
                Expr::BinOp(BinOp::Greater, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == "<=" => {
                Expr::BinOp(BinOp::LessEq, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == ">=" => {
                Expr::BinOp(BinOp::GreaterEq, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }
            [Sexp::Atom(S(op)), e1, e2] if op == "=" => {
                Expr::BinOp(BinOp::Equal, Box::new(parse_expr(e1)), Box::new(parse_expr(e2)))
            }

            [Sexp::Atom(S(kw)), c, t, f] if kw == "if" => Expr::If(
                Box::new(parse_expr(c)),
                Box::new(parse_expr(t)),
                Box::new(parse_expr(f)),
            ),

            [Sexp::Atom(S(kw)), rest @ ..] if kw == "block" => {
                if rest.is_empty() {
                    panic!("block needs at least one expression");
                }
                Expr::Block(rest.iter().map(parse_expr).collect())
            }

            [Sexp::Atom(S(kw)), body] if kw == "loop" => Expr::Loop(Box::new(parse_expr(body))),

            [Sexp::Atom(S(kw)), e] if kw == "break" => Expr::Break(Box::new(parse_expr(e))),

            [Sexp::Atom(S(kw)), Sexp::Atom(S(name)), rhs] if kw == "set!" => {
                if reserved_word(name) {
                    panic!("set! target cannot be keyword: {}", name);
                }
                Expr::Set(name.to_string(), Box::new(parse_expr(rhs)))
            }

            _ => panic!("Invalid expression: {:?}", vec),
        },

        _ => panic!("Invalid expression: {:?}", s),
    }
}

fn mk_label(seq: &mut i32, stem: &str) -> String {
    *seq += 1;
    format!("{}_{}", stem, *seq)
}

fn append_snek_invalid_at(lines: &mut Vec<String>, lab: &str) {
    lines.push(format!("{}:", lab));
    lines.push("mov rdi, 1".to_string());
    lines.push("sub rsp, 8".to_string());
    lines.push("call snek_error".to_string());
}

fn append_snek_overflow_at(lines: &mut Vec<String>, lab: &str) {
    lines.push(format!("{}:", lab));
    lines.push("mov rdi, 2".to_string());
    lines.push("sub rsp, 8".to_string());
    lines.push("call snek_error".to_string());
}

fn append_two_num_checks(depth: i32, lines: &mut Vec<String>, seq: &mut i32) -> String {
    let bad = mk_label(seq, "badarg");
    lines.push("mov rbx, rax".to_string());
    lines.push("and rbx, 1".to_string());
    lines.push("cmp rbx, 0".to_string());
    lines.push(format!("jne {}", bad));
    lines.push(format!("mov rcx, [rsp - {}]", depth));
    lines.push("test rcx, 1".to_string());
    lines.push(format!("jne {}", bad));
    lines.push("mov rdi, rcx".to_string());
    lines.push("sar rdi, 1".to_string());
    lines.push("mov rsi, rax".to_string());
    lines.push("sar rsi, 1".to_string());
    bad
}

fn emit_expr(
    e: &Expr,
    env: &HashMap<String, i32>,
    depth: i32,
    seq: &mut i32,
    exit_loop: Option<&String>,
) -> String {
    match e {
        Expr::Num(n) => {
            let enc = (*n as i64).wrapping_mul(2);
            format!("mov rax, {}", enc)
        }

        Expr::Bool(b) => {
            if *b {
                "mov rax, 3".to_string()
            } else {
                "mov rax, 1".to_string()
            }
        }

        Expr::Input => "mov rax, [rel INPUT_VAL]".to_string(),

        Expr::Var(name) => match env.get(name) {
            Some(off) => format!("mov rax, [rsp - {}]", off),
            None => panic!("Unbound variable: {}", name),
        },

        Expr::Let(bindings, body) => {
            let mut lines = Vec::new();
            let mut next_env = env.clone();
            let mut cursor = depth;
            for (nm, rhs) in bindings {
                lines.push(emit_expr(rhs, &next_env, cursor, seq, exit_loop));
                lines.push(format!("mov [rsp - {}], rax", cursor));
                next_env.insert(nm.clone(), cursor);
                cursor += 8;
            }
            lines.push(emit_expr(body, &next_env, cursor, seq, exit_loop));
            lines.join("\n  ")
        }

        Expr::UnOp(op, sub) => {
            let mut lines = vec![emit_expr(sub, env, depth, seq, exit_loop)];
            match op {
                UnOp::Add1 => {
                    let bad = mk_label(seq, "badarg");
                    let ov = mk_label(seq, "overflow");
                    let done = mk_label(seq, "u_done");
                    lines.push("mov rbx, rax".to_string());
                    lines.push("and rbx, 1".to_string());
                    lines.push("cmp rbx, 0".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push("add rax, 2".to_string());
                    lines.push(format!("jo {}", ov));
                    lines.push(format!("jmp {}", done));
                    append_snek_invalid_at(&mut lines, &bad);
                    append_snek_overflow_at(&mut lines, &ov);
                    lines.push(format!("{}:", done));
                }
                UnOp::Sub1 => {
                    let bad = mk_label(seq, "badarg");
                    let ov = mk_label(seq, "overflow");
                    let done = mk_label(seq, "u_done");
                    lines.push("mov rbx, rax".to_string());
                    lines.push("and rbx, 1".to_string());
                    lines.push("cmp rbx, 0".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push("sub rax, 2".to_string());
                    lines.push(format!("jo {}", ov));
                    lines.push(format!("jmp {}", done));
                    append_snek_invalid_at(&mut lines, &bad);
                    append_snek_overflow_at(&mut lines, &ov);
                    lines.push(format!("{}:", done));
                }
                UnOp::Negate => {
                    let bad = mk_label(seq, "badarg");
                    let ov = mk_label(seq, "overflow");
                    let done = mk_label(seq, "u_done");
                    lines.push("mov rbx, rax".to_string());
                    lines.push("and rbx, 1".to_string());
                    lines.push("cmp rbx, 0".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push("sar rax, 1".to_string());
                    lines.push("neg eax".to_string());
                    lines.push(format!("jo {}", ov));
                    lines.push("movsxd rax, eax".to_string());
                    lines.push("sal rax, 1".to_string());
                    lines.push(format!("jmp {}", done));
                    append_snek_invalid_at(&mut lines, &bad);
                    append_snek_overflow_at(&mut lines, &ov);
                    lines.push(format!("{}:", done));
                }
                UnOp::IsNum => {
                    let t = mk_label(seq, "inum_t");
                    let d = mk_label(seq, "inum_d");
                    lines.push("mov rbx, rax".to_string());
                    lines.push("and rbx, 1".to_string());
                    lines.push("cmp rbx, 0".to_string());
                    lines.push(format!("je {}", t));
                    lines.push("mov rax, 1".to_string());
                    lines.push(format!("jmp {}", d));
                    lines.push(format!("{}:", t));
                    lines.push("mov rax, 3".to_string());
                    lines.push(format!("{}:", d));
                }
                UnOp::IsBool => {
                    let t = mk_label(seq, "ib_t");
                    let d = mk_label(seq, "ib_d");
                    lines.push("mov rbx, rax".to_string());
                    lines.push("and rbx, 1".to_string());
                    lines.push("cmp rbx, 0".to_string());
                    lines.push(format!("jne {}", t));
                    lines.push("mov rax, 1".to_string());
                    lines.push(format!("jmp {}", d));
                    lines.push(format!("{}:", t));
                    lines.push("mov rax, 3".to_string());
                    lines.push(format!("{}:", d));
                }
            }
            lines.join("\n  ")
        }

        Expr::BinOp(op, e1, e2) => {
            let mut lines = Vec::new();
            lines.push(emit_expr(e1, env, depth, seq, exit_loop));
            lines.push(format!("mov [rsp - {}], rax", depth));
            lines.push(emit_expr(e2, env, depth + 8, seq, exit_loop));
            match op {
                BinOp::Plus => {
                    let bad = mk_label(seq, "badarg");
                    let ov = mk_label(seq, "overflow");
                    let done = mk_label(seq, "bin_done");
                    lines.push("mov rbx, rax".to_string());
                    lines.push("and rbx, 1".to_string());
                    lines.push("cmp rbx, 0".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push(format!("mov rbx, [rsp - {}]", depth));
                    lines.push("test rbx, 1".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push(format!("add rax, [rsp - {}]", depth));
                    lines.push(format!("jo {}", ov));
                    lines.push(format!("jmp {}", done));
                    append_snek_invalid_at(&mut lines, &bad);
                    append_snek_overflow_at(&mut lines, &ov);
                    lines.push(format!("{}:", done));
                }
                BinOp::Minus => {
                    let bad = mk_label(seq, "badarg");
                    let ov = mk_label(seq, "overflow");
                    let done = mk_label(seq, "bin_done");
                    lines.push("mov rbx, rax".to_string());
                    lines.push("and rbx, 1".to_string());
                    lines.push("cmp rbx, 0".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push(format!("mov rbx, [rsp - {}]", depth));
                    lines.push("test rbx, 1".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push(format!("mov rcx, [rsp - {}]", depth));
                    lines.push("sub rcx, rax".to_string());
                    lines.push(format!("jo {}", ov));
                    lines.push("mov rax, rcx".to_string());
                    lines.push(format!("jmp {}", done));
                    append_snek_invalid_at(&mut lines, &bad);
                    append_snek_overflow_at(&mut lines, &ov);
                    lines.push(format!("{}:", done));
                }
                BinOp::Times => {
                    let bad = mk_label(seq, "badarg");
                    let ov = mk_label(seq, "overflow");
                    let done = mk_label(seq, "bin_done");
                    lines.push("mov rbx, rax".to_string());
                    lines.push("and rbx, 1".to_string());
                    lines.push("cmp rbx, 0".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push(format!("mov rcx, [rsp - {}]", depth));
                    lines.push("test rcx, 1".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push("mov edi, ecx".to_string());
                    lines.push("sar edi, 1".to_string());
                    lines.push("mov esi, eax".to_string());
                    lines.push("sar esi, 1".to_string());
                    lines.push("mov eax, edi".to_string());
                    lines.push("imul eax, esi".to_string());
                    lines.push(format!("jo {}", ov));
                    lines.push("movsxd rax, eax".to_string());
                    lines.push("sal rax, 1".to_string());
                    lines.push(format!("jmp {}", done));
                    append_snek_invalid_at(&mut lines, &bad);
                    append_snek_overflow_at(&mut lines, &ov);
                    lines.push(format!("{}:", done));
                }
                BinOp::Less => {
                    let bad = append_two_num_checks(depth, &mut lines, seq);
                    let done = mk_label(seq, "bin_done");
                    lines.push("cmp rdi, rsi".to_string());
                    let tr = mk_label(seq, "lt1");
                    let fin = mk_label(seq, "lt2");
                    lines.push(format!("jl {}", tr));
                    lines.push("mov rax, 1".to_string());
                    lines.push(format!("jmp {}", fin));
                    lines.push(format!("{}:", tr));
                    lines.push("mov rax, 3".to_string());
                    lines.push(format!("{}:", fin));
                    lines.push(format!("jmp {}", done));
                    append_snek_invalid_at(&mut lines, &bad);
                    lines.push(format!("{}:", done));
                }
                BinOp::Greater => {
                    let bad = append_two_num_checks(depth, &mut lines, seq);
                    let done = mk_label(seq, "bin_done");
                    lines.push("cmp rdi, rsi".to_string());
                    let tr = mk_label(seq, "gt1");
                    let fin = mk_label(seq, "gt2");
                    lines.push(format!("jg {}", tr));
                    lines.push("mov rax, 1".to_string());
                    lines.push(format!("jmp {}", fin));
                    lines.push(format!("{}:", tr));
                    lines.push("mov rax, 3".to_string());
                    lines.push(format!("{}:", fin));
                    lines.push(format!("jmp {}", done));
                    append_snek_invalid_at(&mut lines, &bad);
                    lines.push(format!("{}:", done));
                }
                BinOp::LessEq => {
                    let bad = append_two_num_checks(depth, &mut lines, seq);
                    let done = mk_label(seq, "bin_done");
                    lines.push("cmp rdi, rsi".to_string());
                    let tr = mk_label(seq, "le1");
                    let fin = mk_label(seq, "le2");
                    lines.push(format!("jle {}", tr));
                    lines.push("mov rax, 1".to_string());
                    lines.push(format!("jmp {}", fin));
                    lines.push(format!("{}:", tr));
                    lines.push("mov rax, 3".to_string());
                    lines.push(format!("{}:", fin));
                    lines.push(format!("jmp {}", done));
                    append_snek_invalid_at(&mut lines, &bad);
                    lines.push(format!("{}:", done));
                }
                BinOp::GreaterEq => {
                    let bad = append_two_num_checks(depth, &mut lines, seq);
                    let done = mk_label(seq, "bin_done");
                    lines.push("cmp rdi, rsi".to_string());
                    let tr = mk_label(seq, "ge1");
                    let fin = mk_label(seq, "ge2");
                    lines.push(format!("jge {}", tr));
                    lines.push("mov rax, 1".to_string());
                    lines.push(format!("jmp {}", fin));
                    lines.push(format!("{}:", tr));
                    lines.push("mov rax, 3".to_string());
                    lines.push(format!("{}:", fin));
                    lines.push(format!("jmp {}", done));
                    append_snek_invalid_at(&mut lines, &bad);
                    lines.push(format!("{}:", done));
                }
                BinOp::Equal => {
                    let bad = mk_label(seq, "badarg");
                    let done = mk_label(seq, "bin_done");
                    lines.push(format!("mov rbx, [rsp - {}]", depth));
                    lines.push("mov rcx, rax".to_string());
                    lines.push("mov rdx, rcx".to_string());
                    lines.push("and rdx, 1".to_string());
                    lines.push("mov rdi, rbx".to_string());
                    lines.push("and rdi, 1".to_string());
                    lines.push("cmp rdx, rdi".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push("cmp rax, rbx".to_string());
                    let tr = mk_label(seq, "eqt");
                    let fin = mk_label(seq, "eqf");
                    lines.push(format!("je {}", tr));
                    lines.push("mov rax, 1".to_string());
                    lines.push(format!("jmp {}", fin));
                    lines.push(format!("{}:", tr));
                    lines.push("mov rax, 3".to_string());
                    lines.push(format!("{}:", fin));
                    lines.push(format!("jmp {}", done));
                    append_snek_invalid_at(&mut lines, &bad);
                    lines.push(format!("{}:", done));
                }
            }
            lines.join("\n  ")
        }

        Expr::If(cond, th, el) => {
            let alt = mk_label(seq, "if_alt");
            let done = mk_label(seq, "if_done");
            let lines = vec![
                emit_expr(cond, env, depth, seq, exit_loop),
                "cmp rax, 1".to_string(),
                format!("je {}", alt),
                emit_expr(th, env, depth, seq, exit_loop),
                format!("jmp {}", done),
                format!("{}:", alt),
                emit_expr(el, env, depth, seq, exit_loop),
                format!("{}:", done),
            ];
            lines.join("\n  ")
        }

        Expr::Block(items) => {
            if items.is_empty() {
                panic!("empty block");
            }
            let mut lines = Vec::new();
            for piece in items {
                lines.push(emit_expr(piece, env, depth, seq, exit_loop));
            }
            lines.join("\n  ")
        }

        Expr::Loop(body) => {
            let head = mk_label(seq, "lp_h");
            let tail = mk_label(seq, "lp_t");
            let lines = vec![
                format!("{}:", head),
                emit_expr(body, env, depth, seq, Some(&tail)),
                format!("jmp {}", head),
                format!("{}:", tail),
            ];
            lines.join("\n  ")
        }

        Expr::Break(inner) => match exit_loop {
            Some(lab) => {
                let lines = vec![
                    emit_expr(inner, env, depth, seq, exit_loop),
                    format!("jmp {}", lab),
                ];
                lines.join("\n  ")
            }
            None => panic!("break outside of loop"),
        },

        Expr::Set(name, rhs) => {
            let off = match env.get(name) {
                Some(o) => *o,
                None => panic!("set! on unknown binding: {}", name),
            };
            let mut lines = vec![emit_expr(rhs, env, depth, seq, exit_loop)];
            lines.push(format!("mov [rsp - {}], rax", off));
            lines.join("\n  ")
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

    let sexp = parse(&in_contents).unwrap_or_else(|e| panic!("Parse error: {}", e));
    let expr = parse_expr(&sexp);

    let env = HashMap::new();
    let mut seq = 0i32;
    let body = emit_expr(&expr, &env, 8, &mut seq, None);

    let mut asm = String::new();
    asm.push_str("section .text\n");
    asm.push_str("default rel\n");
    asm.push_str("extern snek_error\n");
    asm.push_str("extern INPUT_VAL\n");
    asm.push_str("global our_code_starts_here\n");
    asm.push_str("our_code_starts_here:\n");
    asm.push_str("  ");
    asm.push_str(&body);
    asm.push_str("\n  ret\n");

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asm_snippet(e: &Expr) -> String {
        let env = HashMap::new();
        let mut seq = 0i32;
        emit_expr(e, &env, 8, &mut seq, None)
    }

    #[test]
    fn parse_bool_true() {
        let s = parse("true").unwrap();
        assert!(matches!(parse_expr(&s), Expr::Bool(true)));
    }

    #[test]
    fn parse_if_form() {
        let s = parse("(if false 1 2)").unwrap();
        assert!(matches!(parse_expr(&s), Expr::If(_, _, _)));
    }

    #[test]
    fn parse_loop_break() {
        let s = parse("(loop (break 1))").unwrap();
        match parse_expr(&s) {
            Expr::Loop(b) => match *b {
                Expr::Break(_) => {}
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn emit_num_tagged() {
        let a = asm_snippet(&Expr::Num(5));
        assert!(a.contains("mov rax, 10"));
    }

    #[test]
    fn emit_bool_consts() {
        let t = asm_snippet(&Expr::Bool(true));
        assert!(t.contains("mov rax, 3"));
        let f = asm_snippet(&Expr::Bool(false));
        assert!(f.contains("mov rax, 1"));
    }

    #[test]
    fn emit_input_loads_global() {
        let a = asm_snippet(&Expr::Input);
        assert!(a.contains("INPUT_VAL"));
    }

    #[test]
    fn emit_less_compare() {
        let e = Expr::BinOp(
            BinOp::Less,
            Box::new(Expr::Num(1)),
            Box::new(Expr::Num(2)),
        );
        let a = asm_snippet(&e);
        assert!(a.contains("cmp rdi, rsi"));
        assert!(a.contains("jl"));
    }

    #[test]
    fn emit_equal_branch() {
        let e = Expr::BinOp(
            BinOp::Equal,
            Box::new(Expr::Num(3)),
            Box::new(Expr::Num(3)),
        );
        let a = asm_snippet(&e);
        assert!(a.contains("cmp rax, rbx"));
    }

    #[test]
    fn emit_if_jumps() {
        let e = Expr::If(
            Box::new(Expr::Bool(false)),
            Box::new(Expr::Num(1)),
            Box::new(Expr::Num(2)),
        );
        let a = asm_snippet(&e);
        assert!(a.contains("cmp rax, 1"));
        assert!(a.contains("je"));
    }

    #[test]
    fn emit_loop_labels() {
        let e = Expr::Loop(Box::new(Expr::Num(0)));
        let a = asm_snippet(&e);
        assert!(a.contains("lp_h_"));
        assert!(a.contains("lp_t_"));
        assert!(a.contains("jmp"));
    }

    #[test]
    fn emit_break_targets_loop_end() {
        let tail = "lp_t_1".to_string();
        let env = HashMap::new();
        let mut seq = 0i32;
        let a = emit_expr(
            &Expr::Break(Box::new(Expr::Num(7))),
            &env,
            8,
            &mut seq,
            Some(&tail),
        );
        assert!(a.contains(&tail));
    }

    #[test]
    #[should_panic(expected = "break outside of loop")]
    fn break_without_loop_panics() {
        let env = HashMap::new();
        let mut seq = 0i32;
        emit_expr(&Expr::Break(Box::new(Expr::Num(1))), &env, 8, &mut seq, None);
    }

    #[test]
    fn emit_set_store() {
        let e = Expr::Let(
            vec![("x".to_string(), Expr::Num(0))],
            Box::new(Expr::Set(
                "x".to_string(),
                Box::new(Expr::Num(4)),
            )),
        );
        let env = HashMap::new();
        let mut seq = 0i32;
        let a = emit_expr(&e, &env, 8, &mut seq, None);
        assert!(a.contains("[rsp - 8]"));
    }

    #[test]
    fn parse_block_two() {
        let s = parse("(block 1 2)").unwrap();
        match parse_expr(&s) {
            Expr::Block(v) => assert_eq!(v.len(), 2),
            _ => panic!(),
        }
    }

    #[test]
    fn parse_all_cmp_ops() {
        for src in ["(< 1 2)", "(> 1 2)", "(<= 1 2)", "(>= 1 2)"] {
            let s = parse(src).unwrap();
            assert!(matches!(parse_expr(&s), Expr::BinOp(_, _, _)));
        }
    }

    #[test]
    fn parse_isnum_isbool() {
        let s = parse("(isnum 1)").unwrap();
        assert!(matches!(parse_expr(&s), Expr::UnOp(UnOp::IsNum, _)));
        let s = parse("(isbool false)").unwrap();
        assert!(matches!(parse_expr(&s), Expr::UnOp(UnOp::IsBool, _)));
    }

    #[test]
    fn emit_times_uses_imul() {
        let e = Expr::BinOp(
            BinOp::Times,
            Box::new(Expr::Num(2)),
            Box::new(Expr::Num(3)),
        );
        let a = asm_snippet(&e);
        assert!(a.contains("imul"));
    }

    #[test]
    fn nested_if_parses() {
        let s = parse("(if true (if false 1 2) 3)").unwrap();
        assert!(matches!(parse_expr(&s), Expr::If(_, _, _)));
    }

    #[test]
    #[should_panic(expected = "Duplicate binding")]
    fn duplicate_let_rejected() {
        let s = parse("(let ((x 1) (x 2)) x)").unwrap();
        parse_expr(&s);
    }

    #[test]
    #[should_panic]
    fn reserved_id_rejected() {
        let s = parse("(let ((loop 1)) 1)").unwrap();
        parse_expr(&s);
    }

    #[test]
    fn emit_unop_add1_checks_tag() {
        let e = Expr::UnOp(UnOp::Add1, Box::new(Expr::Num(1)));
        let a = asm_snippet(&e);
        assert!(a.contains("add rax, 2"));
    }

    #[test]
    fn emit_negate_path() {
        let e = Expr::UnOp(UnOp::Negate, Box::new(Expr::Num(9)));
        let a = asm_snippet(&e);
        assert!(a.contains("neg eax"));
    }

    #[test]
    fn block_returns_last() {
        let s = parse("(block (add1 0) (add1 1))").unwrap();
        let ex = parse_expr(&s);
        let a = asm_snippet(&ex);
        assert!(a.contains("add rax, 2"));
    }

    #[test]
    fn type_error_plus_bool_emits_snek_error() {
        let e = Expr::BinOp(
            BinOp::Plus,
            Box::new(Expr::Bool(true)),
            Box::new(Expr::Num(1)),
        );
        let a = asm_snippet(&e);
        assert!(a.contains("call snek_error"));
        assert!(a.contains("mov rdi, 1"));
    }

    #[test]
    fn type_error_equal_mixed_tags_emits_snek_error() {
        let e = Expr::BinOp(
            BinOp::Equal,
            Box::new(Expr::Num(1)),
            Box::new(Expr::Bool(false)),
        );
        let a = asm_snippet(&e);
        assert!(a.contains("call snek_error"));
    }

    #[test]
    fn type_error_compare_non_numbers_emits_snek_error() {
        let e = Expr::BinOp(
            BinOp::Less,
            Box::new(Expr::Bool(true)),
            Box::new(Expr::Num(0)),
        );
        let a = asm_snippet(&e);
        assert!(a.contains("call snek_error"));
    }

    #[test]
    fn type_error_add1_on_bool_emits_snek_error() {
        let e = Expr::UnOp(UnOp::Add1, Box::new(Expr::Bool(false)));
        let a = asm_snippet(&e);
        assert!(a.contains("call snek_error"));
    }

    #[test]
    fn overflow_path_add1_emits_overflow_error() {
        let e = Expr::UnOp(UnOp::Add1, Box::new(Expr::Num(1)));
        let a = asm_snippet(&e);
        assert!(a.contains("jo "));
        assert!(a.contains("mov rdi, 2"));
        assert!(a.contains("call snek_error"));
    }

    #[test]
    fn emit_greater_and_greater_eq_use_correct_jumps() {
        let g = Expr::BinOp(
            BinOp::Greater,
            Box::new(Expr::Num(2)),
            Box::new(Expr::Num(1)),
        );
        assert!(asm_snippet(&g).contains("jg"));
        let ge = Expr::BinOp(
            BinOp::GreaterEq,
            Box::new(Expr::Num(1)),
            Box::new(Expr::Num(1)),
        );
        assert!(asm_snippet(&ge).contains("jge"));
    }

    #[test]
    fn parse_readme_loop_example() {
        let s = parse(
            "(let ((x 0)) (loop (if (= x 10) (break x) (set! x (+ x 1)))))",
        )
        .unwrap();
        let ex = parse_expr(&s);
        assert!(matches!(ex, Expr::Let(_, _)));
    }

    #[test]
    fn nested_if_compiles() {
        let s = parse("(if true (if false 1 2) 3)").unwrap();
        let a = asm_snippet(&parse_expr(&s));
        assert!(a.contains("cmp rax, 1"));
        assert!(a.contains("if_alt_"));
    }

    #[test]
    fn set_in_inner_let_compiles() {
        let s = parse("(let ((x 0)) (let ((y 1)) (block (set! x 5) (+ x y))))").unwrap();
        let a = asm_snippet(&parse_expr(&s));
        assert!(a.contains("[rsp -"));
    }

    #[test]
    fn times_emits_overflow_check() {
        let e = Expr::BinOp(
            BinOp::Times,
            Box::new(Expr::Num(1000)),
            Box::new(Expr::Num(1000)),
        );
        let a = asm_snippet(&e);
        assert!(a.contains("imul"));
        assert!(a.contains("jo "));
    }
}
