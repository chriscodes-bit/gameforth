use crate::error::ForthError;
use crate::graphics::Graphics;
use crate::input::InputState;
use crate::memory::Memory;
use crate::rng::Rng;
use crate::stack::{Stack, Value};
use crate::strings::StringPool;
use crate::time::TimeState;

/// All mutable runtime state that a builtin word may touch.
#[derive(Debug)]
pub struct Context {
    pub stack: Stack,
    pub graphics: Graphics,
    pub memory: Memory,
    pub strings: StringPool,
    pub input: InputState,
    pub time: TimeState,
    pub rng: Rng,
    pub return_stack: Vec<i64>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            graphics: Graphics::new(),
            memory: Memory::new(),
            strings: StringPool::new(),
            input: InputState::default(),
            time: TimeState::default(),
            rng: Rng::new(),
            return_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, value: i64) {
        self.stack.push(value);
    }

    pub fn push_value(&mut self, value: Value) {
        self.stack.push_value(value);
    }

    /// Pop a value and require it to be an integer. Floats are rejected
    /// with an error pointing at `int-of`; they are deliberately not
    /// truncated, which would hide mistakes.
    pub fn pop(&mut self) -> Result<i64, ForthError> {
        match self.stack.peek() {
            None => Err(ForthError::new("stack underflow")),
            Some(Value::Float(_)) => Err(ForthError::new(
                "expected an integer, got a float (convert with int-of)",
            )),
            Some(Value::Int(_)) => Ok(self.stack.pop().expect("peeked as int")),
        }
    }

    pub fn pop2(&mut self) -> Result<(i64, i64), ForthError> {
        let right = self.pop()?;
        let left = self.pop()?;
        Ok((left, right))
    }

    /// Pop any value, both integers and floats.
    pub fn pop_value(&mut self) -> Result<Value, ForthError> {
        self.stack
            .pop_value()
            .ok_or_else(|| ForthError::new("stack underflow"))
    }

    /// Pop two values; the pair is `(deeper, top)`.
    pub fn pop2_values(&mut self) -> Result<(Value, Value), ForthError> {
        let right = self.pop_value()?;
        let left = self.pop_value()?;
        Ok((left, right))
    }

    /// Pop a number as an `f64`, promoting integers when needed.
    pub fn pop_number(&mut self) -> Result<f64, ForthError> {
        match self.pop_value()? {
            Value::Int(int) => Ok(int as f64),
            Value::Float(float) => Ok(float),
        }
    }

    pub fn pop2_numbers(&mut self) -> Result<(f64, f64), ForthError> {
        let right = self.pop_number()?;
        let left = self.pop_number()?;
        Ok((left, right))
    }
}
