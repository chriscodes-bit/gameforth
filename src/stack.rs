use std::fmt;

/// A single value on the data stack.
///
/// GameForth's primary type is the 64-bit signed integer. A token that
/// looks like a decimal number (for example `3.14`) lexes as a float and
/// travels the stack as `Value::Float` instead. Arithmetic and comparison
/// words promote int operands to floats when the stack mixes the two;
/// words that only make sense for integers (memory, loops, bitwise work)
/// reject floats with a clear error instead of truncating them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
}

impl Value {
    /// Truth value for `if`, `until`, `while`, and `?dup`.
    pub fn is_truthy(self) -> bool {
        match self {
            Value::Int(int) => int != 0,
            Value::Float(float) => float != 0.0,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(int) => write!(formatter, "{}", int),
            Value::Float(float) => write!(formatter, "{}", float),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Stack {
    data: Vec<Value>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { data: Vec::new() }
    }

    pub fn push(&mut self, value: i64) {
        self.data.push(Value::Int(value));
    }

    pub fn push_value(&mut self, value: Value) {
        self.data.push(value);
    }

    /// Pop a value and require it to be an integer. This does *not*
    /// consume a float; the caller is expected to report the mistake
    /// (see `Context::pop`, which turns this into a proper error), so a
    /// float left on the stack stays there.
    pub fn pop(&mut self) -> Option<i64> {
        if matches!(self.data.last(), Some(Value::Float(_))) {
            return None;
        }

        match self.data.pop() {
            Some(Value::Int(int)) => Some(int),
            _ => None,
        }
    }

    pub fn pop_value(&mut self) -> Option<Value> {
        self.data.pop()
    }

    pub fn peek(&self) -> Option<&Value> {
        self.data.last()
    }

    pub fn second(&self) -> Option<&Value> {
        if self.data.len() < 2 {
            None
        } else {
            self.data.get(self.data.len() - 2)
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn as_slice(&self) -> &[Value] {
        &self.data
    }
}
