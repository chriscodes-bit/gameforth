use crate::error::ForthError;

/// The number of cells used by one Forth cell.
pub const CELL_SIZE: i64 = 1;

/// A flat arena of `i64` cells. Addresses are indices into this arena.
#[derive(Debug, Default, Clone)]
pub struct Memory {
    cells: Vec<i64>,
}

impl Memory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// The address of the next free cell.
    pub fn here(&self) -> usize {
        self.cells.len()
    }

    /// Allocate `count` zeroed cells and return the first address.
    pub fn allocate(&mut self, count: usize) -> usize {
        let address = self.cells.len();
        self.cells.resize(address + count, 0);
        address
    }

    /// Reserve or release cells. Negative counts shrink the arena.
    pub fn allot(&mut self, count: i64) -> Result<(), ForthError> {
        if count >= 0 {
            self.cells.resize(self.cells.len() + count as usize, 0);
            return Ok(());
        }

        let shrink = count.unsigned_abs() as usize;
        if shrink > self.cells.len() {
            return Err(ForthError::new(format!(
                "cannot allot {} cells, only {} available",
                count,
                self.cells.len()
            )));
        }

        self.cells.truncate(self.cells.len() - shrink);
        Ok(())
    }

    /// Append one cell and return its address.
    pub fn push_cell(&mut self, value: i64) -> usize {
        let address = self.cells.len();
        self.cells.push(value);
        address
    }

    pub fn get(&self, address: i64) -> Result<i64, ForthError> {
        let index = self.index(address)?;
        Ok(self.cells[index])
    }

    pub fn set(&mut self, address: i64, value: i64) -> Result<(), ForthError> {
        let index = self.index(address)?;
        self.cells[index] = value;
        Ok(())
    }

    pub fn add_to(&mut self, address: i64, value: i64) -> Result<(), ForthError> {
        let index = self.index(address)?;
        self.cells[index] = self.cells[index].wrapping_add(value);
        Ok(())
    }

    fn index(&self, address: i64) -> Result<usize, ForthError> {
        if address < 0 || address as usize >= self.cells.len() {
            return Err(ForthError::new(format!("invalid address: {}", address)));
        }

        Ok(address as usize)
    }
}
