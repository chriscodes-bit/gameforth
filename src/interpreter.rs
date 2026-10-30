use crate::dictionary::{Dictionary, Word};
use crate::graphics::Graphics;
use crate::lexer::{Token, tokenize};
use crate::stack::Stack;

pub struct Interpreter {
    pub stack: Stack,
    pub graphics: Graphics,
    variables: Vec<i64>,
    dictionary: Dictionary,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            graphics: Graphics::new(),
            variables: Vec::new(),
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
            match &tokens[pc] {
                Token::Number(number) => {
                    self.stack.push(*number);
                }

                Token::Word(word) if word == "if" => {
                    let (else_index, then_index) = find_conditional_boundaries(tokens, pc + 1)?;

                    let condition = self.stack.pop().ok_or("Stack underflow")?;

                    if condition == 0 {
                        pc = else_index.unwrap_or(then_index);
                    }
                }

                Token::Word(word) if word == "else" => {
                    let (_, then_index) = find_conditional_boundaries(tokens, pc + 1)?;

                    pc = then_index;
                }

                Token::Word(word) if word == "then" => {
                    // Nothing to execute; this just marks the end of IF.
                }

                Token::Word(word) if word == "begin" => {
                    // Marks the beginning of a loop.
                }

                Token::Word(word) if word == "until" => {
                    let condition = self.stack.pop().ok_or("Stack underflow")?;

                    if condition == 0 {
                        let begin_index = find_matching_begin(tokens, pc)?;
                        pc = begin_index;
                    }
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

                Token::Word(word) if word == "variable" => {
                    let name = match tokens.get(pc + 1) {
                        Some(Token::Word(name)) => name.clone(),
                        _ => return Err("Expected variable name after variable".to_string()),
                    };

                    // The source is evaluated every frame, so only allocate the
                    // variable the first time it is seen.
                    if !matches!(self.dictionary.find(&name), Some(Word::Variable(_))) {
                        let address = self.variables.len();
                        self.variables.push(0);
                        self.dictionary.define_variable(&name, address);
                    }

                    pc += 1;
                }

                Token::Word(word) if word == "@" => {
                    let address = self.stack.pop().ok_or("Stack underflow")?;
                    let index = self.variable_index(address)?;
                    let value = self.variables[index];
                    self.stack.push(value);
                }

                Token::Word(word) if word == "!" => {
                    let address = self.stack.pop().ok_or("Stack underflow")?;
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let index = self.variable_index(address)?;
                    self.variables[index] = value;
                }

                Token::Word(word) if word == "+!" => {
                    let address = self.stack.pop().ok_or("Stack underflow")?;
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let index = self.variable_index(address)?;
                    self.variables[index] += value;
                }

                Token::Word(word) => match self.dictionary.find(word) {
                    Some(Word::Builtin(function)) => {
                        function(&mut self.stack, &mut self.graphics)?;
                    }
                    Some(Word::UserDefined(body)) => {
                        self.execute_tokens(&body)?;
                    }
                    Some(Word::Color(color)) => {
                        self.graphics.set_color(color);
                    }
                    Some(Word::Variable(address)) => {
                        self.stack.push(address as i64);
                    }
                    None => {
                        return Err(format!("Unknown word: {}", word));
                    }
                },
            }
            /*println!(
                "PC: {}, Token: {:?}, Stack: {:?}",
                pc, tokens[pc], self.stack
            );*/
            pc += 1;
        }

        Ok(())
    }

    fn variable_index(&self, address: i64) -> Result<usize, String> {
        if address < 0 || address as usize >= self.variables.len() {
            return Err(format!("Invalid variable address: {}", address));
        }

        Ok(address as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variables_store_and_fetch() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("variable x 42 x ! x @").unwrap();
        assert_eq!(interpreter.stack.pop(), Some(42));
    }

    #[test]
    fn variable_definitions_are_idempotent() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("variable x 42 x !").unwrap();
        interpreter.eval("variable x x @").unwrap();
        assert_eq!(interpreter.stack.pop(), Some(42));
    }

    #[test]
    fn plus_store_adds_to_variable() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("variable x 10 x ! 5 x +! x @").unwrap();
        assert_eq!(interpreter.stack.pop(), Some(15));
    }

    #[test]
    fn invalid_variable_address_errors() {
        let mut interpreter = Interpreter::new();
        assert!(interpreter.eval("999 @").is_err());
    }

    #[test]
    fn bounce_example_stays_on_screen() {
        let source = std::fs::read_to_string("examples/test.fth").unwrap();
        let mut interpreter = Interpreter::new();

        for _ in 0..500 {
            interpreter.graphics.clear();
            interpreter.eval(&source).unwrap();

            match interpreter.graphics.commands.last().unwrap() {
                crate::graphics::DrawCommand::Rectangle {
                    x,
                    y,
                    width,
                    height,
                    ..
                } => {
                    assert!((0..=700).contains(x));
                    assert!((0..=500).contains(y));
                    assert_eq!(*width, 100);
                    assert_eq!(*height, 100);
                }
                _ => panic!("expected a rectangle"),
            }
        }
    }
}

fn find_conditional_boundaries(
    tokens: &[Token],
    start: usize,
) -> Result<(Option<usize>, usize), String> {
    let mut depth = 0;
    let mut else_index = None;

    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token {
            Token::Word(word) if word == "if" => depth += 1,

            Token::Word(word) if word == "then" => {
                if depth == 0 {
                    return Ok((else_index, index));
                }
                depth -= 1;
            }

            Token::Word(word) if word == "else" && depth == 0 => {
                else_index = Some(index);
            }

            _ => {}
        }
    }

    Err("Missing then for if".to_string())
}

fn find_matching_begin(tokens: &[Token], until_index: usize) -> Result<usize, String> {
    let mut depth = 0;

    for index in (0..until_index).rev() {
        match &tokens[index] {
            Token::Word(word) if word == "until" => {
                depth += 1;
            }

            Token::Word(word) if word == "begin" => {
                if depth == 0 {
                    return Ok(index);
                }

                depth -= 1;
            }

            _ => {}
        }
    }

    Err("Missing begin for until".to_string())
}
