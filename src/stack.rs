#[derive(Debug)]
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

    /*fn dup(&mut self) -> bool {
        match self.peek() {
            Some(&x) => {
                self.push(x);
                true
            }
            None => false,
        }
    }*/

    pub fn is_empty(self) -> bool {
        self.data.is_empty()
    }

    pub fn second(&self) -> Option<&i64> {
        if self.data.len() < 2 {
            None
        } else {
            self.data.get(self.data.len() - 2)
        }
    }
}
