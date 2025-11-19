use std::env;
use std::process;

/// Random number generator state (MWC256)
struct Rng {
    q: [u32; 256],
    carry: u32,
    i: u8,
    initialized: bool,
    seed: u32,
}

impl Rng {
    fn new() -> Self {
        Rng {
            q: [0; 256],
            carry: 362436,
            i: 255,
            initialized: false,
            seed: 123456789,
        }
    }

    fn seed(&mut self, iseed: u32) {
        self.seed = iseed;
        self.initialized = false;
    }

    /// Marsaglia's MWC256 random number generator
    fn next(&mut self) -> u32 {
        const A: u64 = 809430660;

        if !self.initialized {
            self.initialized = true;
            let mut j = self.seed;
            for k in 0..256 {
                j = (69069u32).wrapping_mul(j).wrapping_add(12345);
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
    fn uniform(&mut self) -> f64 {
        const MULT: f64 = 1.0 / 0xFFFFFFFF as f64;
        MULT * self.next() as f64
    }
}

/// Criterion function for CSCV - calculates mean of returns
fn criter(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    returns.iter().sum::<f64>() / returns.len() as f64
}

/// Computes one-bar returns for all short-term and long-term lookbacks
/// of a primitive moving-average crossover system.
fn get_returns(prices: &[f64], max_lookback: usize, returns: &mut Vec<f64>) {
    returns.clear();

    for ilong in 2..=max_lookback {
        for ishort in 1..ilong {
            let mut short_sum = 0.0;
            let mut long_sum = 0.0;

            for i in (max_lookback - 1)..(prices.len() - 1) {
                if i == max_lookback - 1 {
                    // Calculate initial moving averages
                    for j in 0..ishort {
                        short_sum += prices[i - j];
                    }
                    long_sum = short_sum;
                    for j in ishort..ilong {
                        long_sum += prices[i - j];
                    }
                } else {
                    // Update moving averages
                    short_sum += prices[i] - prices[i - ishort];
                    long_sum += prices[i] - prices[i - ilong];
                }

                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;

                let ret = if short_mean > long_mean {
                    // Long position
                    prices[i + 1] - prices[i]
                } else if short_mean < long_mean {
                    // Short position
                    prices[i] - prices[i + 1]
                } else {
                    0.0
                };

                returns.push(ret);
            }
        }
    }

    // Verify computation
    let expected_len = max_lookback * (max_lookback - 1) / 2 * (prices.len() - max_lookback);
    assert_eq!(
        returns.len(),
        expected_len,
        "Returns vector length mismatch"
    );
}

/// External function (already implemented in Rust)
extern "C" {
    fn cscvcore(
        ncases: i32,
        n_systems: i32,
        n_blocks: i32,
        returns: *mut f64,
        indices: *mut i32,
        lengths: *mut i32,
        flags: *mut i32,
        work: *mut f64,
        is_crits: *mut f64,
        oos_crits: *mut f64,
    ) -> f64;
}

fn main() {
    let args: Vec<String> = env::collect();

    // Parse command line arguments
    if args.len() != 6 {
        eprintln!("\nUsage: CSCV  nprices  n_blocks  trend  max_lookback  seed");
        eprintln!("  nprices - number of prices");
        eprintln!("  n_blocks - number of blocks into which cases are partitioned");
        eprintln!("  trend - Amount of trending, 0 for flat system");
        eprintln!("  max_lookback - Maximum moving-average lookback");
        eprintln!("  seed - Random seed, any positive integer");
        process::exit(1);
    }

    let nprices: usize = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing nprices");
        process::exit(1);
    });
    let n_blocks: i32 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing n_blocks");
        process::exit(1);
    });
    let save_trend: f64 = args[3].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing trend");
        process::exit(1);
    });
    let max_lookback: usize = args[4].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing max_lookback");
        process::exit(1);
    });
    let iseed: u32 = args[5].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing seed");
        process::exit(1);
    });

    let n_returns = nprices - max_lookback;
    let n_systems = max_lookback * (max_lookback - 1) / 2;

    // Validate parameters
    if nprices < 2 || n_blocks < 2 || max_lookback < 2 || n_returns < n_blocks as usize {
        eprintln!("\nUsage: CSCV  nprices  n_blocks  trend  max_lookback  seed");
        eprintln!("  nprices - number of prices");
        eprintln!("  n_blocks - number of blocks into which cases are partitioned");
        eprintln!("  trend - Amount of trending, 0 for flat system");
        eprintln!("  max_lookback - Maximum moving-average lookback");
        eprintln!("  seed - Random seed, any positive integer");
        process::exit(1);
    }

    println!(
        "\n\nnprices={}  n_blocks={}  trend={:.3}  max_lookback={}  n_systems={}  n_returns={}",
        nprices, n_blocks, save_trend, max_lookback, n_systems, n_returns
    );

    // Initialize RNG
    let mut rng = Rng::new();
    rng.seed(iseed);

    // Allocate vectors
    let mut prices = vec![0.0; nprices];
    let mut returns = Vec::new();
    let mut indices = vec![0i32; n_blocks as usize];
    let mut lengths = vec![0i32; n_blocks as usize];
    let mut flags = vec![0i32; n_blocks as usize];
    let mut work = vec![0.0; n_returns];
    let mut is_crits = vec![0.0; n_systems];
    let mut oos_crits = vec![0.0; n_systems];

    // Generate log prices
    let mut trend = save_trend;
    prices[0] = 0.0;
    for i in 1..nprices {
        if i % 100 == 0 {
            trend = -trend;
        }
        let rand_sum = rng.uniform() + rng.uniform() - rng.uniform() - rng.uniform();
        prices[i] = prices[i - 1] + trend + rand_sum;
    }

    // Compute returns
    get_returns(&prices, max_lookback, &mut returns);

    // Call cscvcore
    let prob = unsafe {
        cscvcore(
            n_returns as i32,
            n_systems as i32,
            n_blocks,
            returns.as_mut_ptr(),
            indices.as_mut_ptr(),
            lengths.as_mut_ptr(),
            flags.as_mut_ptr(),
            work.as_mut_ptr(),
            is_crits.as_mut_ptr(),
            oos_crits.as_mut_ptr(),
        )
    };

    println!("\nProb = {:.4}", prob);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_seed() {
        let mut rng = Rng::new();
        rng.seed(12345);
        let val1 = rng.next();
        
        let mut rng2 = Rng::new();
        rng2.seed(12345);
        let val2 = rng2.next();
        
        assert_eq!(val1, val2, "Same seed should produce same values");
    }

    #[test]
    fn test_uniform() {
        let mut rng = Rng::new();
        rng.seed(12345);
        for _ in 0..100 {
            let u = rng.uniform();
            assert!(u >= 0.0 && u < 1.0, "uniform() should be in [0, 1)");
        }
    }

    #[test]
    fn test_criter() {
        let returns = vec![1.0, 2.0, 3.0, 4.0];
        assert!((criter(&returns) - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_returns() {
        let prices = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let mut returns = Vec::new();
        get_returns(&prices, 2, &mut returns);
        assert!(!returns.is_empty());
    }
}