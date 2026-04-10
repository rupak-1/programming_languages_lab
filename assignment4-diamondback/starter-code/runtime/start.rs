// runtime/start.rs
// This file provides the entry point for compiled programs

#[link(name = "our_code")]
extern "C" {
    // The \x01 here is an undocumented feature of LLVM that ensures
    // it does not add an underscore in front of the name on macOS
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here() -> i64;
}

#[no_mangle]
pub extern "C" fn snek_error(errcode: i64) {
    if errcode == 1 {
        eprintln!("invalid argument");
    } else if errcode == 2 {
        eprintln!("overflow");
    } else {
        eprintln!("an error occurred ({errcode})");
    }
    std::process::exit(1);
}

#[no_mangle]
pub extern "C" fn snek_print(val: i64) -> i64 {
    if val & 1 == 0 {
        println!("{}", val >> 1);
    } else if val == 3 {
        println!("true");
    } else if val == 1 {
        println!("false");
    } else {
        println!("{val}");
    }
    val
}

#[no_mangle]
pub static mut INPUT_VAL: i64 = 0;

fn render_tagged(v: i64) -> String {
    if v & 1 == 0 {
        format!("{}", v >> 1)
    } else if v == 3 {
        "true".to_string()
    } else if v == 1 {
        "false".to_string()
    } else {
        format!("{v}")
    }
}

fn main() {
    let cli_input: i32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    unsafe {
        INPUT_VAL = (cli_input as i64) << 1;
    }
    let i: i64 = unsafe { our_code_starts_here() };
    println!("{}", render_tagged(i));
}
