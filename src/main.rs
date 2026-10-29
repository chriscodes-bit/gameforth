pub mod dictionary;
pub mod interpreter;
pub mod lexer;
mod stack;

use interpreter::Interpreter;

fn main() {
    let mut forth = Interpreter::new();

    match forth.eval("10 3 - 2 *") {
        Ok(()) => println!("Result: {:?}", forth.stack.peek()),
        Err(error) => println!("Error: {}", error),
    }
}
