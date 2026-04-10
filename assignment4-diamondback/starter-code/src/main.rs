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
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone)]
struct Definition {
    name: String,
    params: Vec<String>,
    body: Expr,
}

#[derive(Debug, Clone)]
struct Program {
    defns: Vec<Definition>,
    main: Expr,
}

#[derive(Debug, Clone)]
enum UnOp {
    Add1,
    Sub1,
    Negate,
    IsNum,
    IsBool,
    Print,
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
            | "print"
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
            | "fun"
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
            [Sexp::Atom(S(op)), e] if op == "print" => {
                Expr::UnOp(UnOp::Print, Box::new(parse_expr(e)))
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

            [Sexp::Atom(S(name)), args @ ..] => {
                if reserved_word(name) {
                    panic!("Invalid expression: {:?}", vec);
                }
                Expr::Call(name.to_string(), args.iter().map(parse_expr).collect())
            }

            _ => panic!("Invalid expression: {:?}", vec),
        },

        _ => panic!("Invalid expression: {:?}", s),
    }
}

fn parse_definition(s: &Sexp) -> Definition {
    match s {
        Sexp::List(items) => match &items[..] {
            [Sexp::Atom(S(fun_kw)), Sexp::List(signature), body] if fun_kw == "fun" => {
                match &signature[..] {
                    [Sexp::Atom(S(name)), params @ ..] => {
                        if reserved_word(name) {
                            panic!("Function name cannot be keyword: {}", name);
                        }
                        let mut seen = HashSet::new();
                        let mut out_params = Vec::new();
                        for p in params {
                            match p {
                                Sexp::Atom(S(param)) => {
                                    if reserved_word(param) {
                                        panic!("Parameter cannot be keyword: {}", param);
                                    }
                                    if !seen.insert(param.clone()) {
                                        panic!("Duplicate parameter: {}", param);
                                    }
                                    out_params.push(param.clone());
                                }
                                _ => panic!("Invalid parameter in function {}", name),
                            }
                        }
                        Definition {
                            name: name.clone(),
                            params: out_params,
                            body: parse_expr(body),
                        }
                    }
                    _ => panic!("Invalid function signature"),
                }
            }
            _ => panic!("Invalid definition"),
        },
        _ => panic!("Invalid definition"),
    }
}

fn is_definition_form(s: &Sexp) -> bool {
    match s {
        Sexp::List(items) => match &items[..] {
            [Sexp::Atom(S(fun_kw)), Sexp::List(_), _] => fun_kw == "fun",
            _ => false,
        },
        _ => false,
    }
}

fn parse_program(s: &Sexp) -> Program {
    match s {
        Sexp::List(items) if items.iter().any(is_definition_form) => {
            if items.is_empty() {
                panic!("Program cannot be empty");
            }
            let mut defns = Vec::new();
            for item in &items[..items.len() - 1] {
                if !is_definition_form(item) {
                    panic!("Function definitions must come before main expression");
                }
                defns.push(parse_definition(item));
            }
            if is_definition_form(&items[items.len() - 1]) {
                panic!("Program must end with a main expression");
            }
            Program {
                defns,
                main: parse_expr(&items[items.len() - 1]),
            }
        }
        _ => Program {
            defns: vec![],
            main: parse_expr(s),
        },
    }
}

fn mk_label(seq: &mut i32, stem: &str) -> String {
    *seq += 1;
    format!("{}_{}", stem, *seq)
}

fn append_snek_invalid_at(lines: &mut Vec<String>, lab: &str) {
    lines.push(format!("{}:", lab));
    lines.push("mov rdi, 1".to_string());
    lines.push("call snek_error".to_string());
}

fn append_snek_overflow_at(lines: &mut Vec<String>, lab: &str) {
    lines.push(format!("{}:", lab));
    lines.push("mov rdi, 2".to_string());
    lines.push("call snek_error".to_string());
}

fn append_two_num_checks(depth: i32, lines: &mut Vec<String>, seq: &mut i32) -> String {
    let bad = mk_label(seq, "badarg");
    lines.push("mov r11, rax".to_string());
    lines.push("and r11, 1".to_string());
    lines.push("cmp r11, 0".to_string());
    lines.push(format!("jne {}", bad));
    lines.push(format!("mov rcx, [rbp - {}]", depth));
    lines.push("test rcx, 1".to_string());
    lines.push(format!("jne {}", bad));
    lines.push("mov rdi, rcx".to_string());
    lines.push("sar rdi, 1".to_string());
    lines.push("mov rsi, rax".to_string());
    lines.push("sar rsi, 1".to_string());
    bad
}

fn load_slot(off: i32) -> String {
    format!("mov rax, [rbp - {}]", off)
}

fn store_slot(off: i32) -> String {
    format!("mov [rbp - {}], rax", off)
}

fn emit_expr(
    e: &Expr,
    env: &HashMap<String, i32>,
    arities: &HashMap<String, usize>,
    param_names: &HashSet<String>,
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
            Some(off) if *off > 0 => load_slot(*off),
            Some(off) => format!("mov rax, [rbp + {}]", -off),
            None => panic!("Unbound variable: {}", name),
        },

        Expr::Let(bindings, body) => {
            let mut lines = Vec::new();
            let mut next_env = env.clone();
            let mut cursor = depth;
            for (nm, rhs) in bindings {
                if param_names.contains(nm) {
                    panic!("Cannot shadow parameter with let: {}", nm);
                }
                lines.push(emit_expr(
                    rhs,
                    &next_env,
                    arities,
                    param_names,
                    cursor,
                    seq,
                    exit_loop,
                ));
                lines.push(store_slot(cursor));
                next_env.insert(nm.clone(), cursor);
                cursor += 8;
            }
            lines.push(emit_expr(
                body,
                &next_env,
                arities,
                param_names,
                cursor,
                seq,
                exit_loop,
            ));
            lines.join("\n  ")
        }

        Expr::UnOp(op, sub) => {
            let mut lines = vec![emit_expr(
                sub,
                env,
                arities,
                param_names,
                depth,
                seq,
                exit_loop,
            )];
            match op {
                UnOp::Add1 => {
                    let bad = mk_label(seq, "badarg");
                    let ov = mk_label(seq, "overflow");
                    let done = mk_label(seq, "u_done");
                    lines.push("mov r11, rax".to_string());
                    lines.push("and r11, 1".to_string());
                    lines.push("cmp r11, 0".to_string());
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
                    lines.push("mov r11, rax".to_string());
                    lines.push("and r11, 1".to_string());
                    lines.push("cmp r11, 0".to_string());
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
                    lines.push("mov r11, rax".to_string());
                    lines.push("and r11, 1".to_string());
                    lines.push("cmp r11, 0".to_string());
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
                    lines.push("mov r11, rax".to_string());
                    lines.push("and r11, 1".to_string());
                    lines.push("cmp r11, 0".to_string());
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
                    lines.push("mov r11, rax".to_string());
                    lines.push("and r11, 1".to_string());
                    lines.push("cmp r11, 0".to_string());
                    lines.push(format!("jne {}", t));
                    lines.push("mov rax, 1".to_string());
                    lines.push(format!("jmp {}", d));
                    lines.push(format!("{}:", t));
                    lines.push("mov rax, 3".to_string());
                    lines.push(format!("{}:", d));
                }
                UnOp::Print => {
                    lines.push("mov rdi, rax".to_string());
                    lines.push("call snek_print".to_string());
                }
            }
            lines.join("\n  ")
        }

        Expr::BinOp(op, e1, e2) => {
            let mut lines = Vec::new();
            lines.push(emit_expr(
                e1,
                env,
                arities,
                param_names,
                depth,
                seq,
                exit_loop,
            ));
            lines.push(store_slot(depth));
            lines.push(emit_expr(
                e2,
                env,
                arities,
                param_names,
                depth + 8,
                seq,
                exit_loop,
            ));
            match op {
                BinOp::Plus => {
                    let bad = mk_label(seq, "badarg");
                    let ov = mk_label(seq, "overflow");
                    let done = mk_label(seq, "bin_done");
                    lines.push("mov r11, rax".to_string());
                    lines.push("and r11, 1".to_string());
                    lines.push("cmp r11, 0".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push(format!("mov r11, [rbp - {}]", depth));
                    lines.push("test r11, 1".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push(format!("add rax, [rbp - {}]", depth));
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
                    lines.push("mov r11, rax".to_string());
                    lines.push("and r11, 1".to_string());
                    lines.push("cmp r11, 0".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push(format!("mov r11, [rbp - {}]", depth));
                    lines.push("test r11, 1".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push(format!("mov rcx, [rbp - {}]", depth));
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
                    lines.push("mov r11, rax".to_string());
                    lines.push("and r11, 1".to_string());
                    lines.push("cmp r11, 0".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push(format!("mov rcx, [rbp - {}]", depth));
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
                    lines.push(format!("mov r11, [rbp - {}]", depth));
                    lines.push("mov rcx, rax".to_string());
                    lines.push("mov rdx, rcx".to_string());
                    lines.push("and rdx, 1".to_string());
                    lines.push("mov rdi, r11".to_string());
                    lines.push("and rdi, 1".to_string());
                    lines.push("cmp rdx, rdi".to_string());
                    lines.push(format!("jne {}", bad));
                    lines.push("cmp rax, r11".to_string());
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
                emit_expr(cond, env, arities, param_names, depth, seq, exit_loop),
                "cmp rax, 1".to_string(),
                format!("je {}", alt),
                emit_expr(th, env, arities, param_names, depth, seq, exit_loop),
                format!("jmp {}", done),
                format!("{}:", alt),
                emit_expr(el, env, arities, param_names, depth, seq, exit_loop),
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
                lines.push(emit_expr(
                    piece,
                    env,
                    arities,
                    param_names,
                    depth,
                    seq,
                    exit_loop,
                ));
            }
            lines.join("\n  ")
        }

        Expr::Loop(body) => {
            let head = mk_label(seq, "lp_h");
            let tail = mk_label(seq, "lp_t");
            let lines = vec![
                format!("{}:", head),
                emit_expr(body, env, arities, param_names, depth, seq, Some(&tail)),
                format!("jmp {}", head),
                format!("{}:", tail),
            ];
            lines.join("\n  ")
        }

        Expr::Break(inner) => match exit_loop {
            Some(lab) => {
                let lines = vec![
                    emit_expr(inner, env, arities, param_names, depth, seq, exit_loop),
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
            let mut lines = vec![emit_expr(
                rhs,
                env,
                arities,
                param_names,
                depth,
                seq,
                exit_loop,
            )];
            if off > 0 {
                lines.push(store_slot(off));
            } else {
                lines.push(format!("mov [rbp + {}], rax", -off));
            }
            lines.join("\n  ")
        }

        Expr::Call(name, args) => {
            let expected = match arities.get(name) {
                Some(arity) => *arity,
                None => panic!("Undefined function: {}", name),
            };
            if expected != args.len() {
                panic!(
                    "Wrong number of arguments in call to {}: expected {}, got {}",
                    name,
                    expected,
                    args.len()
                );
            }
            let mut lines = Vec::new();
            let n = args.len() as i32;
            let eval_depth = depth + n * 8;
            for (i, arg) in args.iter().enumerate() {
                lines.push(emit_expr(
                    arg,
                    env,
                    arities,
                    param_names,
                    eval_depth,
                    seq,
                    exit_loop,
                ));
                lines.push(format!("mov [rbp - {}], rax", depth + (i as i32) * 8));
            }

            let needs_pad = args.len() % 2 == 1;
            if needs_pad {
                lines.push("sub rsp, 8".to_string());
            }
            for i in (0..args.len()).rev() {
                lines.push(format!("mov rax, [rbp - {}]", depth + (i as i32) * 8));
                lines.push("push rax".to_string());
            }
            lines.push(format!("call fun_{}", name));
            let cleanup = (args.len() * 8) + if needs_pad { 8 } else { 0 };
            if cleanup > 0 {
                lines.push(format!("add rsp, {}", cleanup));
            }
            lines.join("\n  ")
        }
    }
}

fn max_stack_depth(e: &Expr, depth: i32) -> i32 {
    match e {
        Expr::Num(_) | Expr::Bool(_) | Expr::Input | Expr::Var(_) => 0,
        Expr::UnOp(_, sub) => max_stack_depth(sub, depth),
        Expr::BinOp(_, e1, e2) => {
            let left = max_stack_depth(e1, depth);
            let right = max_stack_depth(e2, depth + 8);
            left.max(right).max(depth)
        }
        Expr::If(c, t, f) => max_stack_depth(c, depth)
            .max(max_stack_depth(t, depth))
            .max(max_stack_depth(f, depth)),
        Expr::Block(items) => items
            .iter()
            .map(|it| max_stack_depth(it, depth))
            .max()
            .unwrap_or(0),
        Expr::Loop(body) => max_stack_depth(body, depth),
        Expr::Break(inner) => max_stack_depth(inner, depth),
        Expr::Set(_, rhs) => max_stack_depth(rhs, depth),
        Expr::Let(bindings, body) => {
            let mut cursor = depth;
            let mut best = 0;
            for (_, rhs) in bindings {
                best = best.max(max_stack_depth(rhs, cursor)).max(cursor);
                cursor += 8;
            }
            best.max(max_stack_depth(body, cursor))
        }
        Expr::Call(_, args) => {
            let n = args.len() as i32;
            let mut best = if n == 0 { 0 } else { depth + (n - 1) * 8 };
            let eval_depth = depth + n * 8;
            for arg in args {
                best = best.max(max_stack_depth(arg, eval_depth));
            }
            best
        }
    }
}

fn align_to_16(bytes: i32) -> i32 {
    if bytes == 0 {
        0
    } else {
        ((bytes + 15) / 16) * 16
    }
}

fn compile_definition(defn: &Definition, arities: &HashMap<String, usize>, seq: &mut i32) -> String {
    let mut env = HashMap::new();
    for (i, param) in defn.params.iter().enumerate() {
        env.insert(param.clone(), -(16 + (i as i32) * 8));
    }
    let param_names: HashSet<String> = defn.params.iter().cloned().collect();
    let frame_bytes = align_to_16(max_stack_depth(&defn.body, 8));
    let mut lines = vec![
        format!("fun_{}:", defn.name),
        "push rbp".to_string(),
        "mov rbp, rsp".to_string(),
    ];
    if frame_bytes > 0 {
        lines.push(format!("sub rsp, {}", frame_bytes));
    }
    lines.push(emit_expr(
        &defn.body,
        &env,
        arities,
        &param_names,
        8,
        seq,
        None,
    ));
    lines.push("mov rsp, rbp".to_string());
    lines.push("pop rbp".to_string());
    lines.push("ret".to_string());
    lines.join("\n")
}

fn compile_program(prog: &Program) -> String {
    let mut arities = HashMap::new();
    for defn in &prog.defns {
        if arities.insert(defn.name.clone(), defn.params.len()).is_some() {
            panic!("Duplicate function definition: {}", defn.name);
        }
    }

    let mut seq = 0i32;
    let mut lines = vec![
        "section .text".to_string(),
        "default rel".to_string(),
        "extern snek_error".to_string(),
        "extern snek_print".to_string(),
        "extern INPUT_VAL".to_string(),
        "global our_code_starts_here".to_string(),
    ];
    for defn in &prog.defns {
        lines.push(compile_definition(defn, &arities, &mut seq));
    }

    let main_env = HashMap::new();
    let main_params = HashSet::new();
    let main_frame = align_to_16(max_stack_depth(&prog.main, 8));
    lines.push("our_code_starts_here:".to_string());
    lines.push("push rbp".to_string());
    lines.push("mov rbp, rsp".to_string());
    if main_frame > 0 {
        lines.push(format!("sub rsp, {}", main_frame));
    }
    lines.push(emit_expr(
        &prog.main,
        &main_env,
        &arities,
        &main_params,
        8,
        &mut seq,
        None,
    ));
    lines.push("mov rsp, rbp".to_string());
    lines.push("pop rbp".to_string());
    lines.push("ret".to_string());
    format!("{}\n", lines.join("\n"))
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
    let prog = parse_program(&sexp);
    let asm = compile_program(&prog);

    let mut out_file = File::create(out_name)?;
    out_file.write_all(asm.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_prog(src: &str) -> Program {
        parse_program(&parse(src).unwrap())
    }

    fn compile_src(src: &str) -> String {
        compile_program(&parse_prog(src))
    }

    #[test]
    fn parse_single_def_and_main() {
        let p = parse_prog("((fun (id x) x) (id 5))");
        assert_eq!(p.defns.len(), 1);
    }

    #[test]
    fn parse_call_with_zero_args() {
        let p = parse_prog("((fun (forty_two) 42) (forty_two))");
        assert!(matches!(p.main, Expr::Call(_, _)));
    }

    #[test]
    fn parse_call_with_five_args() {
        let p = parse_prog("((fun (sum5 a b c d e) (+ a (+ b (+ c (+ d e))))) (sum5 1 2 3 4 5))");
        assert!(matches!(p.main, Expr::Call(_, _)));
    }

    #[test]
    fn simple_function_call_emits_call_label() {
        let asm = compile_src("((fun (inc x) (add1 x)) (inc 4))");
        assert!(asm.contains("call fun_inc"));
    }

    #[test]
    fn zero_arg_call_emits_no_cleanup() {
        let asm = compile_src("((fun (f) 7) (f))");
        assert!(!asm.contains("add rsp, 0"));
    }

    #[test]
    fn one_arg_call_cleans_up_and_aligns() {
        let asm = compile_src("((fun (f x) x) (f 1))");
        assert!(asm.contains("sub rsp, 8"));
        assert!(asm.contains("add rsp, 16"));
    }

    #[test]
    fn two_arg_call_cleans_exactly_sixteen() {
        let asm = compile_src("((fun (add2 x y) (+ x y)) (add2 1 2))");
        assert!(asm.contains("add rsp, 16"));
    }

    #[test]
    fn five_arg_call_pushes_all_args() {
        let asm = compile_src(
            "((fun (f a b c d e) (+ a (+ b (+ c (+ d e))))) (f 1 2 3 4 5))",
        );
        assert!(asm.matches("push rax").count() >= 5);
    }

    #[test]
    fn recursive_function_compiles_self_call() {
        let asm = compile_src("((fun (fact n) (if (= n 1) 1 (* n (fact (sub1 n))))) (fact 5))");
        assert!(asm.contains("fun_fact:"));
        assert!(asm.contains("call fun_fact"));
    }

    #[test]
    fn mutual_recursion_compiles_both_calls() {
        let asm = compile_src(
            "((fun (is_even n) (if (= n 0) true (is_odd (sub1 n)))) (fun (is_odd n) (if (= n 0) false (is_even (sub1 n)))) (is_even 8))",
        );
        assert!(asm.contains("call fun_is_even"));
        assert!(asm.contains("call fun_is_odd"));
    }

    #[test]
    fn function_calling_function_compiles() {
        let asm = compile_src("((fun (double x) (+ x x)) (fun (quad x) (double (double x))) (quad 3))");
        assert!(asm.contains("call fun_double"));
        assert!(asm.contains("call fun_quad"));
    }

    #[test]
    fn local_variables_use_rbp_negative_offsets() {
        let asm = compile_src("((fun (f x) (let ((y (+ x 1))) (+ y x))) (f 10))");
        assert!(asm.contains("[rbp - 8]"));
    }

    #[test]
    fn parameters_use_rbp_positive_offsets() {
        let asm = compile_src("((fun (f x y) (+ x y)) (f 2 3))");
        assert!(asm.contains("[rbp + 16]"));
        assert!(asm.contains("[rbp + 24]"));
    }

    #[test]
    fn mixed_param_local_access_compiles() {
        let asm = compile_src("((fun (mix x y) (let ((z (+ x y))) (+ z x))) (mix 1 2))");
        assert!(asm.contains("[rbp + 16]"));
        assert!(asm.contains("[rbp - 8]"));
    }

    #[test]
    fn set_can_update_parameter_slot() {
        let asm = compile_src("((fun (f x) (block (set! x (add1 x)) x)) (f 1))");
        assert!(asm.contains("mov [rbp + 16], rax"));
    }

    #[test]
    fn print_unop_calls_runtime_print() {
        let asm = compile_src("(print 5)");
        assert!(asm.contains("call snek_print"));
    }

    #[test]
    fn stack_frame_prologue_and_epilogue_present() {
        let asm = compile_src("((fun (id x) x) (id 1))");
        assert!(asm.contains("push rbp"));
        assert!(asm.contains("mov rbp, rsp"));
        assert!(asm.contains("mov rsp, rbp"));
        assert!(asm.contains("pop rbp"));
    }

    #[test]
    fn main_uses_stack_frame_too() {
        let asm = compile_src("(let ((x 1)) x)");
        assert!(asm.contains("our_code_starts_here:\npush rbp\nmov rbp, rsp"));
    }

    #[test]
    fn parse_plain_expression_program_without_defs() {
        let p = parse_prog("(+ 1 2)");
        assert_eq!(p.defns.len(), 0);
    }

    #[test]
    fn arity_table_rejects_duplicate_definitions() {
        let sexp = parse("((fun (f x) x) (fun (f y) y) (f 1))").unwrap();
        let p = parse_program(&sexp);
        let result = std::panic::catch_unwind(|| compile_program(&p));
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "Wrong number of arguments")]
    fn wrong_number_of_arguments_panics() {
        compile_src("((fun (f x y) (+ x y)) (f 1))");
    }

    #[test]
    #[should_panic(expected = "Undefined function")]
    fn undefined_function_call_panics() {
        compile_src("(missing_fn 1)");
    }

    #[test]
    #[should_panic(expected = "Cannot shadow parameter with let")]
    fn shadowing_parameter_with_let_panics() {
        compile_src("((fun (f x) (let ((x 5)) x)) (f 1))");
    }

    #[test]
    #[should_panic(expected = "Duplicate parameter")]
    fn duplicate_parameter_rejected() {
        parse_prog("((fun (f x x) x) (f 1 2))");
    }

    #[test]
    fn call_argument_order_is_right_to_left_push() {
        let asm = compile_src("((fun (f a b c) a) (f 1 2 3))");
        assert!(asm.contains("call fun_f"));
        assert!(asm.matches("push rax").count() >= 3);
    }

    #[test]
    fn nested_call_uses_temp_slots() {
        let asm = compile_src("((fun (id x) x) (id (id (id 7))))");
        assert!(asm.contains("mov [rbp - 8], rax"));
    }

    #[test]
    fn recursion_with_locals_uses_frame_space() {
        let asm = compile_src(
            "((fun (sum_to n) (if (= n 0) 0 (let ((rest (sum_to (sub1 n)))) (+ n rest)))) (sum_to 4))",
        );
        assert!(asm.contains("sub rsp, "));
        assert!(asm.contains("call fun_sum_to"));
    }

    #[test]
    fn comparison_and_equality_still_emit_runtime_checks() {
        let asm = compile_src("(block (< 1 2) (= 2 2))");
        assert!(asm.contains("cmp rdi, rsi"));
        assert!(asm.contains("call snek_error"));
    }

    #[test]
    fn loop_break_with_calls_compiles() {
        let asm = compile_src("((fun (id x) x) (loop (break (id 5))))");
        assert!(asm.contains("lp_h_"));
        assert!(asm.contains("call fun_id"));
    }

    #[test]
    fn function_definitions_must_precede_main() {
        let sexp = parse("((+ 1 2) (fun (f x) x))").unwrap();
        let result = std::panic::catch_unwind(|| parse_program(&sexp));
        assert!(result.is_err());
    }
}
