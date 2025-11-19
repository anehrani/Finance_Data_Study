use std::env;
use std::process;

/// Random number generator (Marsaglia MWC256)
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

/// Optimization criteria enum
#[derive(Debug, Clone, Copy)]
enum OptCriteria {
    MeanReturn = 0,
    ProfitFactor = 1,
    SharpeRatio = 2,
}

impl OptCriteria {
    fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(OptCriteria::MeanReturn),
            1 => Some(OptCriteria::ProfitFactor),
            2 => Some(OptCriteria::SharpeRatio),
            _ => None,
        }
    }
}

/// Computes optimal short-term and long-term lookbacks for a moving-average crossover system
fn opt_params(
    criteria: OptCriteria,
    x: &[f64],
) -> (f64, usize, usize) {
    let ncases = x.len();
    let mut best_perf = f64::NEG_INFINITY;
    let mut ibestshort = 1;
    let mut ibestlong = 2;

    for ilong in 2..200 {
        for ishort in 1..ilong {
            let mut total_return = 0.0;
            let mut win_sum = 1.0e-60;
            let mut lose_sum = 1.0e-60;
            let mut sum_squares = 1.0e-60;

            let mut short_sum = 0.0;
            let mut long_sum = 0.0;

            for i in (ilong - 1)..(ncases - 1) {
                if i == ilong - 1 {
                    // Calculate initial moving averages
                    for j in 0..ishort {
                        short_sum += x[i - j];
                    }
                    long_sum = short_sum;
                    for j in ishort..ilong {
                        long_sum += x[i - j];
                    }
                } else {
                    // Update moving averages
                    short_sum += x[i] - x[i - ishort];
                    long_sum += x[i] - x[i - ilong];
                }

                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;

                let ret = if short_mean > long_mean {
                    x[i + 1] - x[i]
                } else if short_mean < long_mean {
                    x[i] - x[i + 1]
                } else {
                    0.0
                };

                total_return += ret;
                sum_squares += ret * ret;
                if ret > 0.0 {
                    win_sum += ret;
                } else {
                    lose_sum -= ret;
                }
            }

            // Evaluate based on chosen criteria
            let perf = match criteria {
                OptCriteria::MeanReturn => {
                    let mean = total_return / (ncases - ilong) as f64;
                    mean
                }
                OptCriteria::ProfitFactor => win_sum / lose_sum,
                OptCriteria::SharpeRatio => {
                    let mean = total_return / (ncases - ilong) as f64;
                    let variance = sum_squares / (ncases - ilong) as f64 - mean * mean;
                    let std_dev = variance.sqrt();
                    mean / (std_dev + 1.0e-8)
                }
            };

            if perf > best_perf {
                best_perf = perf;
                ibestshort = ishort;
                ibestlong = ilong;
            }
        }
    }

    (best_perf, ibestshort, ibestlong)
}

/// Tests a trained crossover system
/// Computes the mean return
fn test_system(x: &[f64], short_term: usize, long_term: usize) -> f64 {
    let ncases = x.len();
    let mut sum = 0.0;

    for i in (long_term - 1)..(ncases - 1) {
        // Calculate short-term mean
        let mut short_sum = 0.0;
        for j in 0..short_term {
            short_sum += x[i - j];
        }
        let short_mean = short_sum / short_term as f64;

        // Calculate long-term mean
        let mut long_sum = short_sum;
        for j in short_term..long_term {
            long_sum += x[i - j];
        }
        let long_mean = long_sum / long_term as f64;

        // Take position and cumulate return
        if short_mean > long_mean {
            // Long position
            sum += x[i + 1] - x[i];
        } else if short_mean < long_mean {
            // Short position
            sum -= x[i + 1] - x[i];
        }
    }

    sum / (ncases - long_term) as f64
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    if args.len() != 5 {
        eprintln!("\nUsage: trnbias  which  ncases  trend  nreps");
        eprintln!("  which - 0=mean return  1=profit factor  2=Sharpe ratio");
        eprintln!("  ncases - number of training and test cases");
        eprintln!("  trend - Amount of trending, 0 for flat system");
        eprintln!("  nreps - number of test replications");
        process::exit(1);
    }

    let which: u32 = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing which");
        process::exit(1);
    });

    let which = match OptCriteria::from_u32(which) {
        Some(w) => w,
        None => {
            eprintln!("Error: which must be 0, 1, or 2");
            process::exit(1);
        }
    };

    let ncases: usize = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing ncases");
        process::exit(1);
    });

    let save_trend: f64 = args[3].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing trend");
        process::exit(1);
    });

    let nreps: usize = args[4].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing nreps");
        process::exit(1);
    });

    // Validate parameters
    if ncases < 2 || nreps < 1 {
        eprintln!("\nUsage: trnbias  which  ncases  trend  nreps");
        eprintln!("  which - 0=mean return  1=profit factor  2=Sharpe ratio");
        eprintln!("  ncases - number of training and test cases");
        eprintln!("  trend - Amount of trending, 0 for flat system");
        eprintln!("  nreps - number of test replications");
        process::exit(1);
    }

    println!(
        "\n\nwhich={:?} ncases={} trend={:.3} nreps={}",
        which, ncases, save_trend, nreps
    );

    // Initialize RNG
    let mut rng = Rng::new();
    rng.seed(123456789); // Use default seed

    let mut is_mean = 0.0;
    let mut oos_mean = 0.0;

    // Main replication loop
    for irep in 0..nreps {
        // Generate in-sample set (log prices)
        let mut x = vec![0.0; ncases];
        let mut trend = save_trend;
        x[0] = 0.0;

        for i in 1..ncases {
            if i % 50 == 0 {
                trend = -trend;
            }
            x[i] = x[i - 1]
                + trend
                + rng.uniform()
                + rng.uniform()
                - rng.uniform()
                - rng.uniform();
        }

        // Compute optimal parameters, evaluate return with same dataset
        let (_best_perf, short_lookback, long_lookback) = opt_params(which, &x);
        let is_perf = test_system(&x, short_lookback, long_lookback);

        // Generate out-of-sample set (log prices)
        let mut x_oos = vec![0.0; ncases];
        trend = save_trend;
        x_oos[0] = 0.0;

        for i in 1..ncases {
            if i % 50 == 0 {
                trend = -trend;
            }
            x_oos[i] = x_oos[i - 1]
                + trend
                + rng.uniform()
                + rng.uniform()
                - rng.uniform()
                - rng.uniform();
        }

        // Test the OOS set
        let oos_perf = test_system(&x_oos, short_lookback, long_lookback);

        is_mean += is_perf;
        oos_mean += oos_perf;

        println!(
            "{:3}: {:3} {:3}  {:8.4} {:8.4} ({:8.4})",
            irep,
            short_lookback,
            long_lookback,
            is_perf,
            oos_perf,
            is_perf - oos_perf
        );
    }

    // Print final results
    is_mean /= nreps as f64;
    oos_mean /= nreps as f64;
    let bias = is_mean - oos_mean;

    println!(
        "\nMean IS={:.4}  OOS={:.4}  Bias={:.4}",
        is_mean, oos_mean, bias
    );

    println!("\nPress Enter to exit...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_seed() {
        let mut rng = Rng::new();
        rng.seed(12345);
        let val1 = rng.uniform();

        let mut rng2 = Rng::new();
        rng2.seed(12345);
        let val2 = rng2.uniform();

        assert_eq!(val1, val2);
    }

    #[test]
    fn test_rng_range() {
        let mut rng = Rng::new();
        rng.seed(12345);
        for _ in 0..100 {
            let u = rng.uniform();
            assert!(u >= 0.0 && u < 1.0);
        }
    }

    #[test]
    fn test_opt_params() {
        let mut rng = Rng::new();
        rng.seed(12345);

        let mut x = vec![0.0; 500];
        x[0] = 0.0;
        for i in 1..500 {
            x[i] = x[i - 1] + rng.uniform() - 0.5;
        }

        let (perf, short, long) = opt_params(OptCriteria::MeanReturn, &x);
        assert!(short >= 1);
        assert!(long >= 2);
        assert!(long > short);
        assert!(!perf.is_nan());
    }

    #[test]
    fn test_test_system() {
        let mut x = vec![0.0; 100];
        for i in 1..100 {
            x[i] = x[i - 1] + 0.01;
        }

        let result = test_system(&x, 5, 10);
        assert!(!result.is_nan());
    }

    #[test]
    fn test_opt_criteria_from_u32() {
        assert!(matches!(
            OptCriteria::from_u32(0),
            Some(OptCriteria::MeanReturn)
        ));
        assert!(matches!(
            OptCriteria::from_u32(1),
            Some(OptCriteria::ProfitFactor)
        ));
        assert!(matches!(
            OptCriteria::from_u32(2),
            Some(OptCriteria::SharpeRatio)
        ));
        assert_eq!(OptCriteria::from_u32(3), None);
    }

    #[test]
    fn test_price_generation() {
        let mut rng = Rng::new();
        rng.seed(42);

        let ncases = 100;
        let trend = 0.1;
        let mut x = vec![0.0; ncases];
        x[0] = 0.0;

        for i in 1..ncases {
            x[i] = x[i - 1] + trend + rng.uniform() - 0.5;
        }

        // Verify prices are generated correctly
        assert_eq!(x.len(), ncases);
        assert_eq!(x[0], 0.0);
        for i in 1..ncases {
            assert_ne!(x[i], x[i - 1]); // Should be different due to random component
        }
    }
}