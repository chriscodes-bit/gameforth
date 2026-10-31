/// A small xorshift-based pseudo-random number generator.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Default for Rng {
    fn default() -> Self {
        Self {
            state: 0x9E37_79B9_7F4A_7C15,
        }
    }
}

impl Rng {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed(&mut self, seed: i64) {
        self.state = (seed as u64) ^ 0x9E37_79B9_7F4A_7C15;
        if self.state == 0 {
            self.state = 1;
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.state = value;
        value.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number in `0..upper`. Returns `0` when `upper <= 0`.
    pub fn range(&mut self, upper: i64) -> i64 {
        if upper <= 0 {
            return 0;
        }

        (self.next_u64() % upper as u64) as i64
    }
}
