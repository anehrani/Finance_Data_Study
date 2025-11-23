//! Random number generator (Marsaglia MWC256)

pub struct Rng {
    q: [u32; 256],
    carry: u32,
    i: u8,
    initialized: bool,
    seed: u32,
}

impl Rng {
    pub fn new() -> Self {
        Rng {
            q: [0; 256],
            carry: 362436,
            i: 255,
            initialized: false,
            seed: 123456789,
        }
    }

    pub fn seed(&mut self, iseed: u32) {
        self.seed = iseed;
        self.initialized = false;
    }

    /// Marsaglia's MWC256 random number generator
    pub fn next(&mut self) -> u32 {
        const A: u64 = 809430660;
        if !self.initialized {
            self.initialized = true;
            let mut j = self.seed;
            for k in 0..256 {
                j = 69069u32.wrapping_mul(j).wrapping_add(12345);
                self.q[k] = j;
            }
        }
        self.i = self.i.wrapping_add(1);
        let t: u64 = A * (self.q[self.i as usize] as u64) + (self.carry as u64);
        self.carry = (t >> 32) as u32;
        self.q[self.i as usize] = (t & 0xFFFFFFFF) as u32;
        self.q[self.i as usize]
    }

    /// Returns a random f64 in [0, 1)
    pub fn uniform(&mut self) -> f64 {
        const MULT: f64 = 1.0 / 0xFFFFFFFFu32 as f64;
        MULT * self.next() as f64
    }
}
