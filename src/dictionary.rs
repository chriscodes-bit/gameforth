use crate::stack::Stack;
use std::collections::HashMap;

type Builtin = fn(&mut Stack) -> Result<(), String>;

pub struct Dictionary {
    words: HashMap<String, Builtin>,
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

        dict
    }

    fn add(&mut self, name: &str, function: Builtin) {
        self.words.insert(name.to_string(), function);
    }

    pub fn find(&self, name: &str) -> Option<Builtin> {
        self.words.get(name).copied()
    }
}

fn add(stack: &mut Stack) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;
    stack.push(left + right);
    Ok(())
}

fn subtract(stack: &mut Stack) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;
    stack.push(left - right);
    Ok(())
}

fn multiply(stack: &mut Stack) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;
    stack.push(left * right);
    Ok(())
}

fn divide(stack: &mut Stack) -> Result<(), String> {
    let right = stack.pop().ok_or("Stack underflow")?;
    let left = stack.pop().ok_or("Stack underflow")?;

    if right == 0 {
        return Err("Division by zero".to_string());
    }

    stack.push(left / right);
    Ok(())
}

fn dup(stack: &mut Stack) -> Result<(), String> {
    let value = *stack.peek().ok_or("Stack underflow")?;
    stack.push(value);
    Ok(())
}

fn drop_forth(stack: &mut Stack) -> Result<(), String> {
    stack.pop().ok_or("Stack underflow")?;
    Ok(())
}

fn swap_forth(stack: &mut Stack) -> Result<(), String> {
    let x = stack.pop().ok_or("Stack underflow")?;
    let y = stack.pop().ok_or("Stack underflow")?;
    stack.push(y);
    stack.push(x);
    Ok(())
}
