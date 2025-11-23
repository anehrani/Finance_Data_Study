use std::cell::RefCell;

/// Random number generator based on Marsaglia's MWC256 algorithm
pub struct Rng {
    q: [u32; 256],
    carry: u32,
    i: u8,
}

thread_local! {
    static RNG: RefCell<Rng> = RefCell::new(Rng::new(123456789));
}

impl Rng {
    /// Create a new RNG with the given seed
    pub fn new(seed: u32) -> Self {
        let mut q = [0u32; 256];
        let mut j = seed;
        
        for k in 0..256 {
            j = j.wrapping_mul(69069).wrapping_add(12345);
            q[k] = j;
        }
        
        Rng {
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
    
    /// Generate a random f64 in [0, 1)
    pub fn unifrand(&mut self) -> f64 {
        const MULT: f64 = 1.0 / 0xFFFFFFFFu32 as f64;
        MULT * (self.rand32() as f64)
    }
}

/// Set the seed for the thread-local RNG
pub fn set_seed(seed: u32) {
    RNG.with(|rng| {
        *rng.borrow_mut() = Rng::new(seed);
    });
}

/// Generate a random f64 in [0, 1) using the thread-local RNG
pub fn unifrand() -> f64 {
    RNG.with(|rng| rng.borrow_mut().unifrand())
}

/// Generate a standard normal random variable using Box-Muller method
pub fn normal() -> f64 {
    loop {
        let x1 = unifrand();
        if x1 > 0.0 {
            let x2 = unifrand();
            return (-2.0 * x1.ln()).sqrt() * (2.0 * std::f64::consts::PI * x2).cos();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_deterministic() {
        let mut rng1 = Rng::new(12345);
        let mut rng2 = Rng::new(12345);
        
        for _ in 0..100 {
            assert_eq!(rng1.rand32(), rng2.rand32());
        }
    }

    #[test]
    fn test_unifrand_range() {
        let mut rng = Rng::new(12345);
        for _ in 0..1000 {
            let val = rng.unifrand();
            assert!(val >= 0.0 && val < 1.0);
        }
    }

    #[test]
    fn test_normal_distribution() {
        set_seed(12345);
        let mut sum = 0.0;
        let n = 10000;
        
        for _ in 0..n {
            sum += normal();
        }
        
        let mean = sum / n as f64;
        // Mean should be close to 0
        assert!(mean.abs() < 0.1);
    }
}
