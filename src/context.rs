use crate::error::ForthError;
use crate::graphics::Graphics;
use crate::input::InputState;
use crate::memory::Memory;
use crate::rng::Rng;
use crate::stack::Stack;
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

    pub fn pop(&mut self) -> Result<i64, ForthError> {
        self.stack
            .pop()
            .ok_or_else(|| ForthError::new("stack underflow"))
    }

    pub fn pop2(&mut self) -> Result<(i64, i64), ForthError> {
        let right = self.pop()?;
        let left = self.pop()?;
        Ok((left, right))
    }
}
