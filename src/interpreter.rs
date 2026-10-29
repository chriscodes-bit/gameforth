use crate::dictionary::{Dictionary, Word};
use crate::lexer::{Token, tokenize};
use crate::stack::Stack;

pub struct Interpreter {
    pub stack: Stack,
    dictionary: Dictionary,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            dictionary: Dictionary::new(),
        }
    }

    pub fn eval(&mut self, source: &str) -> Result<(), String> {
        let tokens = tokenize(source);
        self.execute_tokens(&tokens)
    }

    pub fn execute_tokens(&mut self, tokens: &[Token]) -> Result<(), String> {
        let mut pc = 0;

        while pc < tokens.len() {
            println!(
                "PC: {}, Token: {:?}, Stack: {:?}",
                pc, tokens[pc], self.stack
            );
            match &tokens[pc] {
                Token::Number(number) => {
                    self.stack.push(*number);
                }

                Token::Word(word) if word == "if" => {
                    let then_index = find_matching_then(tokens, pc + 1)?;

                    let condition = self.stack.pop().ok_or("Stack underflow")?;

                    println!("IF: condition = {}", condition);
                    println!("IF: then_index = {}", then_index);

                    if condition == 0 {
                        pc = then_index + 1;
                        continue;
                    }

                    println!("IF: executing true branch");
                }

                Token::Word(word) if word == "then" => {
                    // Nothing to execute; this just marks the end of IF.
                }

                Token::Word(word) if word == ":" => {
                    let name = match tokens.get(pc + 1) {
                        Some(Token::Word(name)) => name.clone(),
                        _ => return Err("Expected word name after :".to_string()),
                    };

                    let body_start = pc + 2;

                    let body_end = tokens[body_start..]
                        .iter()
                        .position(|token| matches!(token, Token::Word(w) if w == ";"))
                        .map(|offset| body_start + offset)
                        .ok_or("Missing ; in definition")?;

                    let body = tokens[body_start..body_end].to_vec();

                    self.dictionary.define(&name, body);

                    pc = body_end;
                }

                Token::Word(word) if word == ";" => {
                    return Err("Unexpected ;".to_string());
                }

                Token::Word(word) => match self.dictionary.find(word) {
                    Some(Word::Builtin(function)) => {
                        function(&mut self.stack)?;
                    }
                    Some(Word::UserDefined(body)) => {
                        println!("Executing user word: {:?}", body);
                        self.execute_tokens(&body)?;
                    }
                    /*Some(Word::UserDefined(body)) => {
                        self.execute_tokens(&body)?;
                    }*/
                    None => {
                        return Err(format!("Unknown word: {}", word));
                    }
                },
            }

            pc += 1;
        }

        Ok(())
    }
}

fn find_matching_then(tokens: &[Token], start: usize) -> Result<usize, String> {
    let mut depth = 0;

    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token {
            Token::Word(word) if word == "if" => {
                depth += 1;
            }
            Token::Word(word) if word == "then" => {
                if depth == 0 {
                    return Ok(index);
                }

                depth -= 1;
            }
            _ => {}
        }
    }

    Err("Missing then for if".to_string())
}
