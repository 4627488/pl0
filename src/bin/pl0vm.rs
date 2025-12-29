use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use pl0::vm::VM;
use pl0::tui_interface;
use pl0::types::{Instruction, OpCode};

fn parse_opcode(s: &str) -> OpCode {
    match s {
        "LIT" => OpCode::LIT,
        "OPR" => OpCode::OPR,
        "LOD" => OpCode::LOD,
        "STO" => OpCode::STO,
        "CAL" => OpCode::CAL,
        "INT" => OpCode::INT,
        "JMP" => OpCode::JMP,
        "JPC" => OpCode::JPC,
        "RED" => OpCode::RED,
        "WRT" => OpCode::WRT,
        _ => panic!("Unknown opcode: {}", s),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut file_path = None;
    let mut use_tui = true;

    for arg in args.iter().skip(1) {
        if arg == "--no-tui" {
            use_tui = false;
        } else if !arg.starts_with("--") {
            file_path = Some(arg);
        }
    }

    let path = if let Some(p) = file_path {
        p
    } else {
        eprintln!("Usage: {} <asm_file> [--no-tui]", args[0]);
        std::process::exit(1);
    };

    let file = File::open(path).expect("Failed to open asm file");
    let reader = BufReader::new(file);
    let mut instructions = Vec::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let f = parse_opcode(parts[0]);
            let l = parts[1].parse::<usize>().expect("Failed to parse level");
            let a = parts[2].parse::<i64>().expect("Failed to parse address");
            instructions.push(Instruction::new(f, l, a));
        }
    }

    if !use_tui {
        println!("Loaded {} instructions.", instructions.len());
        println!("Executing...");
    }

    let mut vm = VM::new(instructions);

    if use_tui {
        if let Err(e) = tui_interface::run_tui(vm) {
            eprintln!("Error running TUI: {}", e);
        }
    } else {
        vm.interpret();
    }
}
