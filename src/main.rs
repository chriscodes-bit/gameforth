mod dictionary;
mod interpreter;
mod lexer;
mod stack;

use interpreter::Interpreter;
use std::{env, fs, process};

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: forth <file.fth>");
        process::exit(1);
    });

    let source = fs::read_to_string(&path).unwrap_or_else(|err| {
        eprintln!("Could not read '{}': {}", path, err);
        process::exit(1);
    });

    let mut interpreter = Interpreter::new();

    if let Err(err) = interpreter.eval(&source) {
        eprintln!("Forth error: {}", err);
        process::exit(1);
    }

    println!("Stack: {:?}", interpreter.stack);
}
