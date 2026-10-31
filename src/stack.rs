#[derive(Debug, Default, Clone)]
pub struct Stack {
    data: Vec<i64>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { data: Vec::new() }
    }

    pub fn push(&mut self, value: i64) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Option<i64> {
        self.data.pop()
    }

    pub fn peek(&self) -> Option<&i64> {
        self.data.last()
    }

    pub fn second(&self) -> Option<&i64> {
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

    pub fn as_slice(&self) -> &[i64] {
        &self.data
    }
}
