use std::fmt;

/// A position in a Forth source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(file: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// One entry in a Forth call stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallFrame {
    pub word: String,
    pub location: Option<SourceLocation>,
}

impl CallFrame {
    pub fn new(word: impl Into<String>, location: Option<SourceLocation>) -> Self {
        Self {
            word: word.into(),
            location,
        }
    }
}

/// A runtime or compile-time error with an optional source location and call stack.
#[derive(Debug, Clone)]
pub struct ForthError {
    pub message: String,
    pub location: Option<SourceLocation>,
    pub call_stack: Vec<CallFrame>,
}

impl ForthError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
            call_stack: Vec::new(),
        }
    }

    pub fn at(message: impl Into<String>, location: SourceLocation) -> Self {
        Self {
            message: message.into(),
            location: Some(location),
            call_stack: Vec::new(),
        }
    }

    /// Fill in a location only if the error does not already have one.
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        if self.location.is_none() {
            self.location = Some(location);
        }
        self
    }

    /// Fill in a call stack only if the error does not already have one.
    pub fn with_call_stack(mut self, call_stack: Vec<CallFrame>) -> Self {
        if self.call_stack.is_empty() {
            self.call_stack = call_stack;
        }
        self
    }
}

impl fmt::Display for ForthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.location {
            Some(location) => write!(f, "{}: {}", location, self.message)?,
            None => write!(f, "{}", self.message)?,
        }

        for frame in &self.call_stack {
            match &frame.location {
                Some(location) => write!(f, "\n  in {} at {}", frame.word, location)?,
                None => write!(f, "\n  in {}", frame.word)?,
            }
        }

        Ok(())
    }
}

impl std::error::Error for ForthError {}
