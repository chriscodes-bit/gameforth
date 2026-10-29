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

    fn execute_tokens(&mut self, tokens: &[Token]) -> Result<(), String> {
        let mut i = 0;

        while i < tokens.len() {
            match &tokens[i] {
                Token::Number(n) => self.stack.push(*n),

                Token::Word(word) if word == ":" => {
                    let name = match tokens.get(i + 1) {
                        Some(Token::Word(name)) if name != ":" && name != ";" => name.clone(),
                        _ => return Err("Expected a word name after ':'".to_string()),
                    };

                    let start = i + 2;
                    let end = tokens[start..]
                        .iter()
                        .position(|token| matches!(token, Token::Word(w) if w == ";"))
                        .map(|offset| start + offset)
                        .ok_or("Missing ';' in definition")?;

                    let body = tokens[start..end].to_vec();
                    self.dictionary.define(&name, body);
                    i = end;
                }

                Token::Word(word) if word == ";" => {
                    return Err("Unexpected ';'".to_string());
                }

                Token::Word(word) => match self.dictionary.find(word) {
                    Some(Word::Builtin(function)) => {
                        function(&mut self.stack)?;
                    }
                    Some(Word::UserDefined(body)) => {
                        self.execute_tokens(&body)?;
                    }
                    None => {
                        return Err(format!("Unknown word: {}", word));
                    }
                },
            }

            i += 1;
        }

        Ok(())
    }
}
