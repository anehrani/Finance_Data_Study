use std::env;
use std::io::{self, Read};

const PI: f64 = std::f64::consts::PI;

/*
Normal CDF - Accurate to 7.5 e-8
*/

pub fn normal_cdf(z: f64) -> f64 {
    let zz = z.abs();
    let pdf = (-0.5 * zz * zz).exp() / (2.0 * PI).sqrt();
    let t = 1.0 / (1.0 + zz * 0.2316419);
    let poly = ((((1.330274429 * t - 1.821255978) * t + 1.781477937) * t
        - 0.356563782) * t
        + 0.319381530)
        * t;
    if z > 0.0 {
        1.0 - pdf * poly
    } else {
        pdf * poly
    }
}



/*
Random number generator - Marsaglia's MWC256
*/

pub struct RNG {
    q: [u32; 256],
    carry: u32,
    initialized: bool,
    seed: u32,
    i: u8,
}

impl RNG {
    pub fn new() -> Self {
        RNG {
            q: [0; 256],
            carry: 362436,
            initialized: false,
            seed: 123456789,
            i: 255,
        }
    }

    pub fn seed(&mut self, iseed: u32) {
        self.seed = iseed;
        self.initialized = false;
    }

    pub fn rand32m(&mut self) -> u32 {
        if !self.initialized {
            self.initialized = true;
            let mut j = self.seed;
            for k in 0..256 {
                j = j.wrapping_mul(69069).wrapping_add(12345);
                self.q[k] = j;
            }
        }

        self.i = self.i.wrapping_add(1);
        let t: u64 = (809430660u64)
            .wrapping_mul(self.q[self.i as usize] as u64)
            .wrapping_add(self.carry as u64);
        self.carry = (t >> 32) as u32;
        self.q[self.i as usize] = (t & 0xFFFFFFFF) as u32;
        self.q[self.i as usize]
    }

    pub fn unifrand(&mut self) -> f64 {
        self.rand32m() as f64 / 0xFFFFFFFFu32 as f64
    }
}

/*
Compute a single indicator (linear slope of a price block) 
and a single target (price change over a specified lookahead)
*/

pub fn ind_targ(
    lookback: usize,
    lookahead: usize,
    x: &[f64],
    x_idx: usize, // Index into x array for current price
) -> (f64, f64) {
    let start_idx = if x_idx >= lookback - 1 {
        x_idx - lookback + 1
    } else {
        0
    };

    let mut slope = 0.0;
    let mut denom = 0.0;

    for i in 0..lookback {
        let coef = 2.0 * i as f64 / (lookback - 1) as f64 - 1.0;
        denom += coef * coef;
        slope += coef * x[start_idx + i];
    }

    let indicator = slope / denom;
    let target = x[x_idx + lookahead] - x[x_idx];
    (indicator, target)
}

/*
Compute beta coefficient for simple linear regression
*/

pub fn find_beta(data: &[(f64, f64)]) -> (f64, f64) {
    let ntrn = data.len();
    if ntrn == 0 {
        return (0.0, 0.0);
    }

    let mut xmean = 0.0;
    let mut ymean = 0.0;

    for &(x, y) in data {
        xmean += x;
        ymean += y;
    }

    xmean /= ntrn as f64;
    ymean /= ntrn as f64;

    let mut xy = 0.0;
    let mut xx = 0.0;

    for &(x, y) in data {
        let dx = x - xmean;
        let dy = y - ymean;
        xy += dx * dy;
        xx += dx * dx;
    }

    let beta = xy / (xx + 1e-60);
    let constant = ymean - beta * xmean;
    (beta, constant)
}

/*
Main routine
*/


// fn main() {
//     let args: Vec<String> = env::args().collect();

//     if args.len() != 9 {
//         eprintln!("Usage: overlap <nprices> <lookback> <lookahead> <ntrain> <ntest> <omit> <extra> <nreps>");
//         eprintln!("  nprices - Total number of prices (bars in history)");
//         eprintln!("  lookback - historical window length for indicator");
//         eprintln!("  lookahead - Bars into future for target");
//         eprintln!("  ntrain - Number of cases in training set");
//         eprintln!("  ntest - Number of cases in test set");
//         eprintln!("  omit - Omit this many cases from end of training window");
//         eprintln!("  extra - Extra (beyond ntest) bars jumped for next fold");
//         eprintln!("  nreps - Number of replications");
//         std::process::exit(1);
//     }

//     let nprices: usize = args[1].parse().expect("Invalid nprices");
//     let lookback: usize = args[2].parse().expect("Invalid lookback");
//     let lookahead: usize = args[3].parse().expect("Invalid lookahead");
//     let ntrain: usize = args[4].parse().expect("Invalid ntrain");
//     let ntest: usize = args[5].parse().expect("Invalid ntest");
//     let omit: usize = args[6].parse().expect("Invalid omit");
//     let extra: usize = args[7].parse().expect("Invalid extra");
//     let mut nreps: usize = args[8].parse().expect("Invalid nreps");

//     // Force nreps to be odd
//     nreps = nreps / 2 * 2 + 1;

//     if nprices < 2
//         || lookback < 2
//         || lookahead < 1
//         || ntrain < 2
//         || ntest < 1
//         || nprices < lookback + lookahead + ntrain + ntest + 10
//     {
//         eprintln!("Nprices must be at least lookback + lookahead + ntrain + ntest + 10");
//         eprintln!("Usage: overlap <nprices> <lookback> <lookahead> <ntrain> <ntest> <omit> <extra> <nreps>");
//         std::process::exit(1);
//     }

//     println!(
//         "\n\nnprices={}  lookback={}  lookahead={}  ntrain={}  ntest={}  omit={}  extra={}",
//         nprices, lookback, lookahead, ntrain, ntest, omit, extra
//     );

//     let mut rng = RNG::new();
//     let mut save_t = vec![0.0; nreps];
//     let mut p1_count = 0;

//     /*
//     This replicates the test a few times in one run to get median t and p<=0.1 count.
//     */

//     for irep in 0..nreps {
//         /*
//         Generate the log prices as a random walk,
//         and then compute the dataset (indicator, target pairs)
//         */

//         let mut x = vec![0.0; nprices];
//         for i in 1..nprices {
//             x[i] = x[i - 1] + rng.unifrand() + rng.unifrand() - rng.unifrand() - rng.unifrand();
//         }

//         let mut data = Vec::new();
//         for i in (lookback - 1)..(nprices - lookahead) {
//             let (ind, targ) = ind_targ(lookback, lookahead, &x, i);
//             data.push((ind, targ));
//         }

//         let ncases = data.len();

//         /*
//         Compute the walkforward OOS values
//         */

//         let mut oos = Vec::new();
//         let mut istart = ntrain;
//         let mut trn_idx = 0;

//         loop {
//             let test_idx = trn_idx + ntrain;
//             if test_idx >= ncases {
//                 break;
//             }

//             // Get training data (omit last 'omit' cases)
//             let training_size = if ntrain > omit { ntrain - omit } else { 1 };
//             let training_data: Vec<(f64, f64)> =
//                 data[trn_idx..(trn_idx + training_size)].to_vec();

//             let (beta, constant) = find_beta(&training_data);

//             let nt = if ntest > ncases - istart {
//                 ncases - istart
//             } else {
//                 ntest
//             };

//             // Test on the next nt cases
//             for itest in 0..nt {
//                 let test_case_idx = test_idx + itest;
//                 if test_case_idx >= ncases {
//                     break;
//                 }

//                 let (indicator, target) = data[test_case_idx];
//                 let pred = beta * indicator + constant;

//                 if pred > 0.0 {
//                     oos.push(target);
//                 } else {
//                     oos.push(-target);
//                 }
//             }

//             istart += nt + extra;
//             trn_idx += nt + extra;
//         }

//         let n_oos = oos.len();

//         /*
//         Analyze results
//         */

//         let mut oos_mean = 0.0;
//         let mut oos_ss = 0.0;

//         for &val in &oos {
//             oos_mean += val;
//             oos_ss += val * val;
//         }

//         oos_mean /= n_oos as f64;
//         oos_ss /= n_oos as f64;
//         oos_ss -= oos_mean * oos_mean; // Variance

//         if oos_ss < 1e-20 {
//             oos_ss = 1e-20;
//         }

//         let t = (n_oos as f64).sqrt() * oos_mean / oos_ss.sqrt();
//         let rtail = 1.0 - normal_cdf(t);

//         println!(
//             "\nMean = {:.4}  StdDev = {:.4}  t = {:.4}  p = {:.4}",
//             oos_mean,
//             oos_ss.sqrt(),
//             t,
//             rtail
//         );

//         save_t[irep] = t;

//         if rtail <= 0.1 {
//             p1_count += 1;
//         }
//     } // For all replications

//     qsortd(&mut save_t);
//     println!(
//         "\nn OOS = {}  Median t = {:.4}  Fraction with p<= 0.1 = {:.3}",
//         save_t.len(),
//         save_t[nreps / 2],
//         p1_count as f64 / nreps as f64
//     );

//     println!("\nPress Enter to exit...");
//     let _ = io::stdin().read(&mut [0u8]);
// }
