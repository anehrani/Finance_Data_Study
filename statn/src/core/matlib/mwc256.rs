/// MWC256 random number generator
/// This is a random int generator suggested by Marsaglia in his DIEHARD suite.
/// It provides a great combination of speed and quality.
pub struct Mwc256 {
    q: [u32; 256],
    carry: u32,
    i: u8,
}

impl Mwc256 {
    /// Create a new random number generator with a default seed
    pub fn new() -> Self {
        Self::with_seed(123456789)
    }

    /// Create a new random number generator with the given seed
    pub fn with_seed(seed: u32) -> Self {
        let mut q = [0u32; 256];
        let mut j = seed;
        
        for q_val in &mut q {
            j = j.wrapping_mul(69069).wrapping_add(12345);
            *q_val = j;
        }
        
        Self {
            q,
            carry: 362436,
            i: 255,
        }
    }
    
    /// Generate a random u32
    pub fn rand32(&mut self) -> u32 {
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
        MULT * (self.rand32() as f64)
    }

    /// Generate a standard normal random variable using Box-Muller method
    pub fn normal(&mut self) -> f64 {
        loop {
            let x1 = self.unifrand();
            if x1 > 0.0 {
                let x2 = self.unifrand();
                return (-2.0 * x1.ln()).sqrt() * (2.0 * std::f64::consts::PI * x2).cos();
            }
        }
    }
}

impl Default for Mwc256 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reproducibility() {
        let mut rng1 = Mwc256::with_seed(42);
        let mut rng2 = Mwc256::with_seed(42);
        
        for _ in 0..100 {
            assert_eq!(rng1.rand32(), rng2.rand32());
        }
    }
    
    #[test]
    fn test_unifrand_range() {
        let mut rng = Mwc256::with_seed(12345);
        
        for _ in 0..1000 {
            let val = rng.unifrand();
            assert!(val >= 0.0 && val < 1.0);
        }
    }
    
    #[test]
    fn test_different_seeds() {
        let mut rng1 = Mwc256::with_seed(42);
        let mut rng2 = Mwc256::with_seed(43);
        
        let val1 = rng1.rand32();
        let val2 = rng2.rand32();
        
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_normal() {
        let mut rng = Mwc256::with_seed(12345);
        for _ in 0..1000 {
            let val = rng.normal();
            assert!(val.is_finite());
        }
    }
}
