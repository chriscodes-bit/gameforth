use crate::dictionary::Dictionary;
use crate::lexer::{Token, tokenize};
use crate::stack::Stack;

pub struct Interpreter {
    pub stack: Stack,
    dictionary: Dictionary,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            stack: Stack::new(),
            dictionary: Dictionary::new(),
        }
    }

    pub fn eval(&mut self, source: &str) -> Result<(), String> {
        let tokens = tokenize(source);

        for token in tokens {
            match token {
                Token::Number(n) => {
                    self.stack.push(n);
                }

                Token::Word(word) => {
                    let function = self
                        .dictionary
                        .find(&word)
                        .ok_or_else(|| format!("Unknown word: {}", word))?;

                    function(&mut self.stack)?;
                }
            }
        }

        Ok(())
    }
}
