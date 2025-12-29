mod lexer;
mod parser;
mod tui_interface;
mod types;
mod vm;

use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    // Simple arg parsing
    let mut file_path = None;
    let mut use_tui = true;

    for arg in args.iter().skip(1) {
        if arg == "--no-tui" {
            use_tui = false;
        } else if !arg.starts_with("--") {
            file_path = Some(arg);
        }
    }

    let source_code = if let Some(path) = file_path {
        fs::read_to_string(path).expect("Failed to read file")
    } else {
        // Default test program
        r#"
        program gcd;
        var x, y;
        begin
            x := 36;
            y := 24;
            while x # y do
                if x > y then x := x - y
                else y := y - x;
            write(x)
        end.
        "#
        .to_string()
    };

    if !use_tui {
        println!("Source Code:\n{}", source_code);
    }

    let lexer = Lexer::new(&source_code);
    let mut parser = Parser::new(lexer);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        parser.parse();
    }));

    if let Err(_) = result {
        println!("Compilation failed.");
        return;
    }

    if !use_tui {
        println!("Compilation successful!");
        println!("Generated Code:");
        for (i, instr) in parser.code.iter().enumerate() {
            println!("{:3}: {:?} {}, {}", i, instr.f, instr.l, instr.a);
        }
    }

    let mut vm = VM::new(parser.code);

    if use_tui {
        if let Err(e) = tui_interface::run_tui(vm) {
            eprintln!("Error running TUI: {}", e);
        }
    } else {
        vm.interpret();
    }
}
