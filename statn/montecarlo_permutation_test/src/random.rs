/// MWC256 random number generator
/// This is a random int generator suggested by Marsaglia in his DIEHARD suite.
/// It provides a great combination of speed and quality.

pub struct Rand32M {
    q: [u32; 256],
    carry: u32,
    i: u8,
}

impl Rand32M {
    /// Create a new random number generator with the given seed
    pub fn new(seed: u32) -> Self {
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
    
    /// Generate a random f64 in the range [0, 1)
    pub fn unifrand(&mut self) -> f64 {
        const MULT: f64 = 1.0 / 0xFFFFFFFFu32 as f64;
        MULT * (self.rand32m() as f64)
    }
}

impl Default for Rand32M {
    fn default() -> Self {
        Self::new(123456789)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reproducibility() {
        let mut rng1 = Rand32M::new(42);
        let mut rng2 = Rand32M::new(42);
        
        for _ in 0..100 {
            assert_eq!(rng1.rand32m(), rng2.rand32m());
        }
    }
    
    #[test]
    fn test_unifrand_range() {
        let mut rng = Rand32M::new(12345);
        
        for _ in 0..1000 {
            let val = rng.unifrand();
            assert!(val >= 0.0 && val < 1.0);
        }
    }
    
    #[test]
    fn test_different_seeds() {
        let mut rng1 = Rand32M::new(42);
        let mut rng2 = Rand32M::new(43);
        
        let val1 = rng1.rand32m();
        let val2 = rng2.rand32m();
        
        assert_ne!(val1, val2);
    }
}
