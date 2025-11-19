use std::env;
use std::io::{self, Read};

/*
Random number generator - Marsaglia's MWC256
*/

struct RNG {
    q: [u32; 256],
    carry: u32,
    initialized: bool,
    seed: u32,
    i: u8,
}

impl RNG {
    fn new() -> Self {
        RNG {
            q: [0; 256],
            carry: 362436,
            initialized: false,
            seed: 123456789,
            i: 255,
        }
    }

    fn seed(&mut self, iseed: u32) {
        self.seed = iseed;
        self.initialized = false;
    }

    fn rand32m(&mut self) -> u32 {
        if !self.initialized {
            self.initialized = true;
            let mut j = self.seed;
            for k in 0..256 {
                j = j.wrapping_mul(69069).wrapping_add(12345);
                self.q[k] = j;
            }
        }

        self.i = self.i.wrapping_add(1);
        let t: u64 = (809430660u64).wrapping_mul(self.q[self.i as usize] as u64)
            .wrapping_add(self.carry as u64);
        self.carry = (t >> 32) as u32;
        self.q[self.i as usize] = (t & 0xFFFFFFFF) as u32;
        self.q[self.i as usize]
    }

    fn unifrand(&mut self) -> f64 {
        self.rand32m() as f64 / 0xFFFFFFFF as f64
    }
}

/*
Compute optimal short-term and long-term lookbacks for moving-average crossover system
*/

fn opt_params(
    which: usize,        // 0=mean return; 1=profit factor; 2=Sharpe ratio
    long_v_short: bool,  // true=long only, false=short-only
    ncases: usize,
    x: &[f64],
    rng: &mut RNG,
) -> (usize, usize, f64) {
    let mut best_perf = -1e60;
    let mut ibestshort = 1;
    let mut ibestlong = 2;

    for ilong in 2..200 {
        for ishort in 1..ilong {
            let mut total_return = 0.0;
            let mut win_sum = 1e-60;
            let mut lose_sum = 1e-60;
            let mut sum_squares = 1e-60;
            let mut n_trades = 0;

            let mut short_sum = 0.0;
            let mut long_sum = 0.0;

            for i in (ilong - 1)..(ncases - 1) {
                if i == ilong - 1 {
                    short_sum = 0.0;
                    for j in (i - ishort + 1)..=i {
                        short_sum += x[j];
                    }
                    long_sum = short_sum;
                    for j in (i - ilong + 1)..(i - ishort + 1) {
                        long_sum += x[j];
                    }
                } else {
                    short_sum += x[i] - x[i - ishort];
                    long_sum += x[i] - x[i - ilong];
                }

                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;

                let (traded, ret) = if long_v_short && short_mean > long_mean {
                    (true, x[i + 1] - x[i])
                } else if !long_v_short && short_mean < long_mean {
                    (true, x[i] - x[i + 1])
                } else {
                    (false, 0.0)
                };

                if traded {
                    n_trades += 1;
                    total_return += ret;
                    sum_squares += ret * ret;
                    if ret > 0.0 {
                        win_sum += ret;
                    } else {
                        lose_sum -= ret;
                    }
                }
            }

            let n_trades_f = n_trades as f64 + 1e-30;

            let perf = match which {
                0 => {
                    // Mean return criterion
                    total_return / n_trades_f
                }
                1 => {
                    // Profit factor criterion
                    win_sum / lose_sum
                }
                2 => {
                    // Sharpe ratio criterion
                    let mean_ret = total_return / n_trades_f;
                    let mut variance = sum_squares / n_trades_f - mean_ret * mean_ret;
                    if variance < 1e-20 {
                        variance = 1e-20;
                    }
                    mean_ret / variance.sqrt()
                }
                _ => 0.0,
            };

            if perf > best_perf {
                best_perf = perf;
                ibestshort = ishort;
                ibestlong = ilong;
            }
        }
    }

    (ibestshort, ibestlong, best_perf)
}

/*
Test a trained crossover system and compute mean return
*/

fn test_system(
    long_v_short: bool,
    ncases: usize,
    x: &[f64],
    short_term: usize,
    long_term: usize,
) -> f64 {
    let mut sum = 0.0;
    let mut n_trades = 0;

    for i in (long_term - 1)..(ncases - 1) {
        let mut short_mean = 0.0;
        for j in (i - short_term + 1)..=i {
            short_mean += x[j];
        }
        short_mean /= short_term as f64;

        let mut long_mean = short_mean;
        for j in (i - long_term + 1)..(i - short_term + 1) {
            long_mean += x[j];
        }
        long_mean /= long_term as f64;

        if long_v_short && short_mean > long_mean {
            sum += x[i + 1] - x[i];
            n_trades += 1;
        } else if !long_v_short && short_mean < long_mean {
            sum += x[i] - x[i + 1];
            n_trades += 1;
        }
    }

    sum / (n_trades as f64 + 1e-30)
}

/*
Main routine
*/

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        eprintln!("Usage: selbias <which> <ncases> <trend> <nreps>");
        eprintln!("  which - 0=mean return  1=profit factor  2=Sharpe ratio");
        eprintln!("  ncases - number of training and test cases");
        eprintln!("  trend - Amount of trending, 0 for flat system");
        eprintln!("  nreps - number of test replications");
        std::process::exit(1);
    }

    let which: usize = args[1].parse().expect("Invalid which");
    let ncases: usize = args[2].parse().expect("Invalid ncases");
    let save_trend: f64 = args[3].parse().expect("Invalid trend");
    let nreps: usize = args[4].parse().expect("Invalid nreps");

    if ncases < 2 || which > 2 || nreps < 1 {
        eprintln!("Usage: selbias <which> <ncases> <trend> <nreps>");
        eprintln!("  which - 0=mean return  1=profit factor  2=Sharpe ratio");
        eprintln!("  ncases - number of training and test cases");
        eprintln!("  trend - Amount of trending, 0 for flat system");
        eprintln!("  nreps - number of test replications");
        std::process::exit(1);
    }

    println!("\nwhich={} ncases={} trend={:.3} nreps={}", which, ncases, save_trend, nreps);

    let mut rng = RNG::new();
    let mut x = vec![0.0; ncases];

    let mut l_is_mean = 0.0;
    let mut s_is_mean = 0.0;
    let mut l_oos_mean = 0.0;
    let mut s_oos_mean = 0.0;
    let mut oos_mean = 0.0;
    let mut bias_mean = 0.0;
    let mut bias_ss = 0.0;

    for irep in 0..nreps {
        // Generate in-sample set
        let mut trend = save_trend;
        x[0] = 0.0;
        for i in 1..ncases {
            if (i + 1) % 50 == 0 {
                trend = -trend;
            }
            x[i] = x[i - 1] + trend + rng.unifrand() + rng.unifrand() - rng.unifrand() - rng.unifrand();
        }

        // Compute optimal parameters for long and short models
        let (l_short_lookback, l_long_lookback, _) = opt_params(which, true, ncases, &x, &mut rng);
        let l_is_perf = test_system(true, ncases, &x, l_short_lookback, l_long_lookback);

        let (s_short_lookback, s_long_lookback, _) = opt_params(which, false, ncases, &x, &mut rng);
        let s_is_perf = test_system(false, ncases, &x, s_short_lookback, s_long_lookback);

        // Generate first OOS set
        trend = save_trend;
        x[0] = 0.0;
        for i in 1..ncases {
            if (i + 1) % 50 == 0 {
                trend = -trend;
            }
            x[i] = x[i - 1] + trend + rng.unifrand() + rng.unifrand() - rng.unifrand() - rng.unifrand();
        }

        let l_oos_perf = test_system(true, ncases, &x, l_short_lookback, l_long_lookback);
        let s_oos_perf = test_system(false, ncases, &x, s_short_lookback, s_long_lookback);

        l_is_mean += l_is_perf;
        l_oos_mean += l_oos_perf;
        s_is_mean += s_is_perf;
        s_oos_mean += s_oos_perf;

        println!(
            "{:3}: {:3} {:3} {:3} {:3}  {:8.4} {:8.4} ({:8.4})  {:8.4} {:8.4} ({:8.4})",
            irep,
            l_short_lookback,
            l_long_lookback,
            s_short_lookback,
            s_long_lookback,
            l_is_perf,
            l_oos_perf,
            l_is_perf - l_oos_perf,
            s_is_perf,
            s_oos_perf,
            s_is_perf - s_oos_perf
        );

        // Generate second OOS set
        trend = save_trend;
        x[0] = 0.0;
        for i in 1..ncases {
            if (i + 1) % 50 == 0 {
                trend = -trend;
            }
            x[i] = x[i - 1] + trend + rng.unifrand() + rng.unifrand() - rng.unifrand() - rng.unifrand();
        }

        // Choose model based on first OOS set performance
        let (oos_perf, bias) = if l_oos_perf > s_oos_perf {
            let perf = test_system(true, ncases, &x, l_short_lookback, l_long_lookback);
            (perf, l_oos_perf - perf)
        } else {
            let perf = test_system(false, ncases, &x, s_short_lookback, s_long_lookback);
            (perf, s_oos_perf - perf)
        };

        bias_mean += bias;
        bias_ss += bias * bias;
        oos_mean += oos_perf;

        println!("     OOS_perf={:8.4}  Bias={:8.4}", oos_perf, bias);
    }

    // Compute averages
    l_is_mean /= nreps as f64;
    l_oos_mean /= nreps as f64;
    s_is_mean /= nreps as f64;
    s_oos_mean /= nreps as f64;

    println!(
        "\n\nLong training bias = {:.4}  short = {:.4}",
        l_is_mean - l_oos_mean,
        s_is_mean - s_oos_mean
    );

    oos_mean /= nreps as f64;
    bias_mean /= nreps as f64;
    bias_ss /= nreps as f64;
    bias_ss -= bias_mean * bias_mean;
    if bias_ss < 1e-20 {
        bias_ss = 1e-20;
    }
    let t = (nreps as f64).sqrt() * bias_mean / bias_ss.sqrt();

    println!(
        "OOS={:.4}  Selection bias={:.4}  t={:.3}",
        oos_mean, bias_mean, t
    );

    println!("\nPress Enter to exit...");
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}