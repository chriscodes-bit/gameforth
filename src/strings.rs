use crate::error::ForthError;

/// Interned strings. Forth code sees an integer handle, not a Rust string.
#[derive(Debug, Default, Clone)]
pub struct StringPool {
    values: Vec<String>,
}

impl StringPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, value: String) -> i64 {
        self.values.push(value);
        (self.values.len() - 1) as i64
    }

    pub fn get(&self, handle: i64) -> Result<&str, ForthError> {
        if handle < 0 || handle as usize >= self.values.len() {
            return Err(ForthError::new(format!(
                "invalid string handle: {}",
                handle
            )));
        }

        Ok(&self.values[handle as usize])
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
