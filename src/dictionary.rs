use crate::graphics::Graphics;
use crate::lexer::Token;
use crate::stack::Stack;
use std::collections::HashMap;

pub type Builtin = fn(&mut Stack, &mut Graphics) -> Result<(), String>;

#[derive(Clone)]
pub enum Word {
    Builtin(Builtin),
    UserDefined(Vec<Token>),
}

pub struct Dictionary {
    words: HashMap<String, Word>,
}

impl Dictionary {
    pub fn new() -> Self {
        let mut dict = Dictionary {
            words: HashMap::new(),
        };

        dict.add("+", add);
        dict.add("-", subtract);
        dict.add("*", multiply);
        dict.add("/", divide);

        dict.add("dup", dup);
        dict.add("drop", drop_forth);
        dict.add("swap", swap_forth);
        dict.add("over", over);

        dict.add("=", equal_forth);
        dict.add("<", less_than_forth);
        dict.add(">", greater_than_forth);
        dict.add("0=", zero_equal_forth);

        dict.add(".", print_forth);

        // Graphics
        dict.add("draw-rectangle", draw_rectangle);

        dict
    }

    fn add(&mut self, name: &str, function: Builtin) {
        self.words.insert(name.to_string(), Word::Builtin(function));
    }

    pub fn define(&mut self, name: &str, body: Vec<Token>) {
        self.words.insert(name.to_string(), Word::UserDefined(body));
    }

    pub fn find(&self, name: &str) -> Option<Word> {
        self.words.get(name).cloned()
    }
}

// Arithmetic

fn add(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;
    stack.push(left + right);
    Ok(())
}

fn subtract(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;
    stack.push(left - right);
    Ok(())
}

fn multiply(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;
    stack.push(left * right);
    Ok(())
}

fn divide(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;

    if right == 0 {
        return Err("Division by zero".to_string());
    }

    stack.push(left / right);
    Ok(())
}

// Stack operations

fn dup(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let value = *stack.peek().ok_or("Stack underflow")?;
    stack.push(value);
    Ok(())
}

fn drop_forth(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    stack.pop().ok_or("Stack underflow")?;
    Ok(())
}

fn swap_forth(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;

    stack.push(right);
    stack.push(left);

    Ok(())
}

fn over(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let value = *stack.second().ok_or("Stack underflow")?;
    stack.push(value);
    Ok(())
}

// Comparison operations

fn equal_forth(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;

    stack.push(if left == right { -1 } else { 0 });
    Ok(())
}

fn less_than_forth(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;

    stack.push(if left < right { -1 } else { 0 });
    Ok(())
}

fn greater_than_forth(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;

    stack.push(if left > right { -1 } else { 0 });
    Ok(())
}

fn zero_equal_forth(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let value = stack.pop().ok_or("Stack underflow")?;

    stack.push(if value == 0 { -1 } else { 0 });
    Ok(())
}

// Output

fn print_forth(stack: &mut Stack, _graphics: &mut Graphics) -> Result<(), String> {
    let value = stack.pop().ok_or("Stack underflow")?;

    println!("{}", value);

    Ok(())
}

// Graphics

fn draw_rectangle(stack: &mut Stack, graphics: &mut Graphics) -> Result<(), String> {
    let height = stack.pop().ok_or("Stack underflow")?;
    let width = stack.pop().ok_or("Stack underflow")?;
    let y = stack.pop().ok_or("Stack underflow")?;
    let x = stack.pop().ok_or("Stack underflow")?;

    graphics.draw_rectangle(x as i32, y as i32, width as i32, height as i32);

    Ok(())
}
