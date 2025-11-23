/// Marsaglia's MWC256 random number generator
/// Provides a great combination of speed and quality
pub struct Rng {
    q: [u32; 256],
    carry: u32,
    i: u8,
}

impl Rng {
    /// Create a new RNG with default seed
    pub fn new() -> Self {
        Self::with_seed(123456789)
    }

    /// Create a new RNG with specified seed
    pub fn with_seed(seed: u32) -> Self {
        let mut q = [0u32; 256];
        let mut j = seed;
        
        for k in 0..256 {
            j = j.wrapping_mul(69069).wrapping_add(12345);
            q[k] = j;
        }

        Self {
            q,
            carry: 362436,
            i: 255,
        }
    }

    /// Generate a random u32
    pub fn rand32m(&mut self) -> u32 {
        const A: u64 = 809430660;
        
        self.i = self.i.wrapping_add(1);
        let t = A * (self.q[self.i as usize] as u64) + (self.carry as u64);
        self.carry = (t >> 32) as u32;
        self.q[self.i as usize] = (t & 0xFFFFFFFF) as u32;
        self.q[self.i as usize]
    }

    /// Generate a random f64 in [0, 1)
    pub fn unifrand(&mut self) -> f64 {
        const MULT: f64 = 1.0 / 0xFFFFFFFFu32 as f64;
        MULT * (self.rand32m() as f64)
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_deterministic() {
        let mut rng1 = Rng::with_seed(12345);
        let mut rng2 = Rng::with_seed(12345);
        
        for _ in 0..100 {
            assert_eq!(rng1.rand32m(), rng2.rand32m());
        }
    }

    #[test]
    fn test_unifrand_range() {
        let mut rng = Rng::new();
        
        for _ in 0..1000 {
            let val = rng.unifrand();
            assert!(val >= 0.0 && val < 1.0);
        }
    }
}
