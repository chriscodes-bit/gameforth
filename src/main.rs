pub mod dictionary;
pub mod interpreter;
pub mod lexer;
mod stack;

use interpreter::Interpreter;

fn main() {
    let mut forth = Interpreter::new();

    match forth.eval(
        ": square dup * ; \
         : double 2 * ; \
         5 square \
         3 double",
    ) {
        Ok(()) => println!("Stack: {:?}", forth.stack),
        Err(error) => println!("Error: {}", error),
    }
}
