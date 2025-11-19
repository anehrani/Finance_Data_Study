use std::env;
use std::process;

const PI: f64 = std::f64::consts::PI;

/// Normal CDF - Accurate to 7.5e-8
fn normal_cdf(z: f64) -> f64 {
    let zz = z.abs();
    let pdf = (-0.5 * zz * zz).exp() / (2.0 * PI).sqrt();
    let t = 1.0 / (1.0 + zz * 0.2316419);
    let poly = ((((1.330274429 * t - 1.821255978) * t + 1.781477937) * t - 0.356563782) * t
        + 0.319381530)
        * t;
    if z > 0.0 {
        1.0 - pdf * poly
    } else {
        pdf * poly
    }
}

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

/// Computes a single indicator (linear slope) and target (price change)
fn ind_targ(lookback: usize, lookahead: usize, x: &[f64], idx: usize) -> (f64, f64) {
    let mut slope = 0.0;
    let mut denom = 0.0;

    for i in 0..lookback {
        let coef = 2.0 * i as f64 / (lookback as f64 - 1.0) - 1.0;
        denom += coef * coef;
        slope += coef * x[idx - lookback + 1 + i];
    }

    let ind = slope / denom;
    let targ = x[idx + lookahead] - x[idx];

    (ind, targ)
}

/// Computes beta coefficient for simple linear regression
fn find_beta(ntrn: usize, data: &[(f64, f64)]) -> (f64, f64) {
    let mut xmean = 0.0;
    let mut ymean = 0.0;

    for (x, y) in data.iter().take(ntrn) {
        xmean += x;
        ymean += y;
    }

    xmean /= ntrn as f64;
    ymean /= ntrn as f64;

    let mut xy = 0.0;
    let mut xx = 0.0;

    for (x, y) in data.iter().take(ntrn) {
        let x_centered = x - xmean;
        let y_centered = y - ymean;
        xy += x_centered * y_centered;
        xx += x_centered * x_centered;
    }

    let beta = xy / (xx + 1.0e-60);
    let constant = ymean - beta * xmean;

    (beta, constant)
}

/// Quicksort for f64 array
fn qsort(data: &mut [f64]) {
    if data.len() <= 1 {
        return;
    }
    qsort_recursive(data, 0, data.len() - 1);
}

fn qsort_recursive(data: &mut [f64], first: usize, last: usize) {
    let split = data[(first + last) / 2];
    let mut lower = first;
    let mut upper = last;

    loop {
        while data[lower] < split {
            lower += 1;
        }
        while data[upper] > split {
            if upper == 0 {
                break;
            }
            upper -= 1;
        }

        if lower > upper {
            break;
        }

        data.swap(lower, upper);
        if lower < upper {
            lower += 1;
            upper = if upper > 0 { upper - 1 } else { 0 };
        } else {
            break;
        }
    }

    if first < upper {
        qsort_recursive(data, first, upper);
    }
    if lower < last {
        qsort_recursive(data, lower, last);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    if args.len() != 11 {
        eprintln!("\nUsage: xvw  nprices  trend  lookback  lookahead  ntrain  ntest  nfolds  omit  nreps  seed");
        eprintln!("  nprices - Total number of prices (bars in history)");
        eprintln!("  trend - Amount of trending, 0 for pure random walk");
        eprintln!("  lookback - historical window length for indicator");
        eprintln!("  lookahead - Bars into future for target");
        eprintln!("  ntrain - Number of cases in training set");
        eprintln!("  ntest - Number of cases in test set");
        eprintln!("  nfolds - Number of XVAL folds");
        eprintln!("  omit - Omit this many cases from end of training window");
        eprintln!("  nreps - Number of replications");
        eprintln!("  seed - Random seed");
        process::exit(1);
    }

    let nprices: usize = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing nprices");
        process::exit(1);
    });
    let trend: f64 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing trend");
        process::exit(1);
    });
    let lookback: usize = args[3].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing lookback");
        process::exit(1);
    });
    let lookahead: usize = args[4].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing lookahead");
        process::exit(1);
    });
    let ntrain: usize = args[5].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing ntrain");
        process::exit(1);
    });
    let ntest: usize = args[6].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing ntest");
        process::exit(1);
    });
    let mut nfolds: usize = args[7].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing nfolds");
        process::exit(1);
    });
    let omit: usize = args[8].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing omit");
        process::exit(1);
    });
    let nreps: usize = args[9].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing nreps");
        process::exit(1);
    });
    let seed: u32 = args[10].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing seed");
        process::exit(1);
    });

    // Validate parameters
    if nprices < 2
        || lookback < 2
        || lookahead < 1
        || ntrain < 2
        || ntest < 1
        || nfolds < 2
        || omit > ntrain
        || nprices < lookback + lookahead + ntrain + ntest + 10
    {
        eprintln!("\nUsage: xvw  nprices  trend  lookback  lookahead  ntrain  ntest  nfolds  omit  nreps  seed");
        eprintln!("  nprices must be at least lookback + lookahead + ntrain + ntest + 10");
        process::exit(1);
    }

    println!(
        "\n\nnprices={}  trend={:.3}  lookback={}  lookahead={}  ntrain={}  ntest={}  nfolds={}  omit={}  nreps={}  seed={}",
        nprices, trend, lookback, lookahead, ntrain, ntest, nfolds, omit, nreps, seed
    );

    let save_trend = trend;
    let mut rng = Rng::new();
    rng.seed(seed);

    let mut mean_w = 0.0;
    let mut mean_x = 0.0;
    let mut ss_w = 0.0;
    let mut ss_x = 0.0;

    // Replicate the test many times
    for irep in 0..nreps {
        println!("\n{:.2}%", 100.0 * irep as f64 / nreps as f64);

        // Generate log prices as random walk
        let mut x = vec![0.0; nprices];
        let mut current_trend = save_trend;

        for i in 1..nprices {
            if (i + 1) % 50 == 0 {
                current_trend = -current_trend;
            }
            x[i] = x[i - 1]
                + current_trend
                + rng.uniform()
                + rng.uniform()
                - rng.uniform()
                - rng.uniform();
        }

        // Compute dataset (indicator, target) pairs
        let mut data: Vec<(f64, f64)> = Vec::new();
        for i in (lookback - 1)..(nprices - lookahead) {
            let (ind, targ) = ind_targ(lookback, lookahead, &x, i);
            data.push((ind, targ));
        }

        let ncases = data.len();
        let mut data_save = if omit > 0 { data.clone() } else { vec![] };

        // Limit nfolds to ncases
        let mut nfolds_adj = nfolds;
        if nfolds_adj > ncases {
            eprintln!(
                "Number of XVAL folds reduced from {} to {}",
                nfolds, ncases
            );
            nfolds_adj = ncases;
        }

        // ===== Compute walkforward OOS values =====
        let mut oos_walk: Vec<f64> = Vec::new();
        let mut istart = ntrain;

        let mut trn_ptr = 0;
        let mut ifold = 0;

        loop {
            let test_ptr = trn_ptr + ntrain;
            if test_ptr >= ncases {
                break;
            }

            let (beta, constant) = find_beta(ntrain - omit, &data[trn_ptr..]);
            let mut nt = ntest;
            if nt > ncases - istart {
                nt = ncases - istart;
            }

            for itest in 0..nt {
                let idx = test_ptr + itest;
                if idx < ncases {
                    let (ind, targ) = data[idx];
                    let pred = beta * ind + constant;
                    if pred > 0.0 {
                        oos_walk.push(targ);
                    } else {
                        oos_walk.push(-targ);
                    }
                }
            }

            istart += nt;
            trn_ptr += nt;
            ifold += 1;
        }

        let oos_mean_w = if !oos_walk.is_empty() {
            oos_walk.iter().sum::<f64>() / oos_walk.len() as f64
        } else {
            0.0
        };

        println!("WALK n OOS = {}  Mean = {:.4}", oos_walk.len(), oos_mean_w);

        // ===== Compute XVAL OOS values =====
        let mut oos_xval: Vec<f64> = Vec::new();
        let ncases_save = ncases;
        let mut current_ncases = ncases;
        let mut n_done = 0;

        for ifold in 0..nfolds_adj {
            let n_in_fold = (ncases_save - n_done) / (nfolds_adj - ifold);
            let istart_fold = n_done;
            let istop_fold = istart_fold + n_in_fold;

            // Prepare training data based on omit parameter
            let mut train_data: Vec<(f64, f64)> = Vec::new();

            if omit > 0 {
                if ifold == 0 {
                    // First fold: skip omit cases after OOS
                    for i in (istop_fold + omit)..ncases_save {
                        train_data.push(data_save[i]);
                    }
                } else if ifold == nfolds_adj - 1 {
                    // Last fold: skip omit cases before OOS
                    for i in 0..(istart_fold.saturating_sub(omit)) {
                        train_data.push(data_save[i]);
                    }
                } else {
                    // Interior fold: skip omit cases on both sides
                    for i in 0..(istart_fold.saturating_sub(omit)) {
                        train_data.push(data_save[i]);
                    }
                    for i in (istop_fold + omit)..ncases_save {
                        train_data.push(data_save[i]);
                    }
                }
            } else {
                // No omit: include all cases except OOS fold
                for i in 0..istart_fold {
                    train_data.push(data[i]);
                }
                for i in istop_fold..ncases_save {
                    train_data.push(data[i]);
                }
            }

            if train_data.is_empty() {
                continue;
            }

            let (beta, constant) = find_beta(train_data.len(), &train_data);

            // Test on OOS fold
            for i in istart_fold..istop_fold {
                let (ind, targ) = if omit > 0 {
                    data_save[i]
                } else {
                    data[i]
                };
                let pred = beta * ind + constant;
                if pred > 0.0 {
                    oos_xval.push(targ);
                } else {
                    oos_xval.push(-targ);
                }
            }

            n_done += n_in_fold;
        }

        let oos_mean_x = if !oos_xval.is_empty() {
            oos_xval.iter().sum::<f64>() / oos_xval.len() as f64
        } else {
            0.0
        };

        println!("XVAL n OOS = {}  Mean = {:.4}", oos_xval.len(), oos_mean_x);

        // Cumulate statistics for t-test
        mean_w += oos_mean_w;
        mean_x += oos_mean_x;
        ss_w += oos_mean_w * oos_mean_w;
        ss_x += oos_mean_x * oos_mean_x;
    }

    // Final computation
    mean_w /= nreps as f64;
    mean_x /= nreps as f64;

    let var_w = ss_w / nreps as f64 - mean_w * mean_w;
    let var_x = ss_x / nreps as f64 - mean_x * mean_x;

    let denom = (var_w + var_x) / (nreps as f64 * (nreps as f64 - 1.0));
    let denom = denom.sqrt();

    let t = (mean_x - mean_w) / denom;
    let p_value = 1.0 - normal_cdf(t);

    let t_w = (nreps as f64).sqrt() * mean_w / var_w.sqrt();
    let t_x = (nreps as f64).sqrt() * mean_x / var_x.sqrt();

    println!(
        "\n\nnprices={}  trend={:.3}  lookback={}  lookahead={}  ntrain={}  ntest={}  nfolds={}  omit={}  nreps={}  seed={}",
        nprices, save_trend, lookback, lookahead, ntrain, ntest, nfolds, omit, nreps, seed
    );

    println!(
        "\n\nGrand XVAL = {:.5} (t={:.3})  WALK = {:.5} (t={:.3})  StdDev = {:.5}  t = {:.3}  rtail = {:.5}",
        mean_x, t_x, mean_w, t_w, denom, t, p_value
    );

    println!("\nPress Enter to exit...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_cdf() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-7);
        assert!(normal_cdf(5.0) > 0.999);
        assert!(normal_cdf(-5.0) < 0.001);
    }

    #[test]
    fn test_rng() {
        let mut rng = Rng::new();
        rng.seed(12345);
        let val1 = rng.uniform();
        assert!(val1 >= 0.0 && val1 < 1.0);
    }

    #[test]
    fn test_rng_deterministic() {
        let mut rng1 = Rng::new();
        rng1.seed(12345);
        let val1 = rng1.uniform();

        let mut rng2 = Rng::new();
        rng2.seed(12345);
        let val2 = rng2.uniform();

        assert_eq!(val1, val2);
    }

    #[test]
    fn test_ind_targ() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let (ind, targ) = ind_targ(3, 2, &x, 4);
        assert!(!ind.is_nan());
        assert!(!targ.is_nan());
    }

    #[test]
    fn test_find_beta() {
        let data = vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)];
        let (beta, constant) = find_beta(3, &data);
        assert!((beta - 2.0).abs() < 1e-10);
        assert!(constant.abs() < 1e-10);
    }

    #[test]
    fn test_qsort() {
        let mut data = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0];
        qsort(&mut data);
        assert_eq!(data, vec![1.0, 1.0, 3.0, 4.0, 5.0, 9.0]);
    }
}