use clap::Parser;
use std::f64::consts::PI;

/// Explore the effect of unobvious IS/OOS overlap in walkforward
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Total number of prices (bars in history)
    nprices: usize,

    /// Historical window length for indicator
    lookback: usize,

    /// Bars into future for target
    lookahead: usize,

    /// Number of cases in training set
    ntrain: usize,

    /// Number of cases in test set
    ntest: usize,

    /// Omit this many cases from end of training window
    omit: usize,

    /// Extra (beyond ntest) bars jumped for next fold
    extra: usize,

    /// Number of replications
    nreps: usize,
}

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

/// In-place quicksort for f64 slices
fn quicksort(data: &mut [f64]) {
    if data.len() <= 1 {
        return;
    }
    quicksort_range(data, 0, data.len() - 1);
}

fn quicksort_range(data: &mut [f64], first: usize, last: usize) {
    if first >= last {
        return;
    }

    let split = data[(first + last) / 2];
    let mut lower = first;
    let mut upper = last;

    loop {
        while split > data[lower] {
            lower += 1;
        }
        while split < data[upper] {
            upper = upper.saturating_sub(1);
        }

        if lower == upper {
            lower += 1;
            upper = upper.saturating_sub(1);
        } else if lower < upper {
            data.swap(lower, upper);
            lower += 1;
            upper = upper.saturating_sub(1);
        }

        if lower > upper {
            break;
        }
    }

    if first < upper {
        quicksort_range(data, first, upper);
    }
    if lower < last {
        quicksort_range(data, lower, last);
    }
}

/// Marsaglia's MWC256 random number generator
struct Mwc256 {
    q: [u32; 256],
    carry: u32,
    i: u8,
}

impl Mwc256 {
    fn new(seed: u32) -> Self {
        let mut q = [0u32; 256];
        let mut j = seed;

        for item in q.iter_mut() {
            j = j.wrapping_mul(69069).wrapping_add(12345);
            *item = j;
        }

        Mwc256 {
            q,
            carry: 362436,
            i: 255,
        }
    }

    fn next(&mut self) -> u32 {
        self.i = self.i.wrapping_add(1);
        let a: u64 = 809430660;
        let t = a * (self.q[self.i as usize] as u64) + (self.carry as u64);
        self.carry = (t >> 32) as u32;
        self.q[self.i as usize] = (t & 0xFFFFFFFF) as u32;
        self.q[self.i as usize]
    }

    fn unifrand(&mut self) -> f64 {
        let mult = 1.0 / 0xFFFFFFFFu32 as f64;
        mult * (self.next() as f64)
    }
}

/// Compute indicator (linear slope) and target (price change)
fn ind_targ(lookback: usize, lookahead: usize, x: &[f64]) -> (f64, f64) {
    let mut slope = 0.0;
    let mut denom = 0.0;

    for i in 0..lookback {
        let coef = 2.0 * (i as f64) / ((lookback - 1) as f64) - 1.0;
        denom += coef * coef;
        slope += coef * x[i];
    }

    let ind = slope / denom;
    let targ = x[lookback - 1 + lookahead] - x[lookback - 1];

    (ind, targ)
}

/// Find beta coefficient for simple linear regression
fn find_beta(data: &[(f64, f64)]) -> (f64, f64) {
    let n = data.len() as f64;
    let mut xmean = 0.0;
    let mut ymean = 0.0;

    for (x, y) in data.iter() {
        xmean += x;
        ymean += y;
    }

    xmean /= n;
    ymean /= n;

    let mut xy = 0.0;
    let mut xx = 0.0;

    for (x, y) in data.iter() {
        let x_dev = x - xmean;
        let y_dev = y - ymean;
        xy += x_dev * y_dev;
        xx += x_dev * x_dev;
    }

    let beta = xy / (xx + 1e-60);
    let constant = ymean - beta * xmean;

    (beta, constant)
}

fn main() {
    let mut args = Args::parse();

    // Force nreps to be odd
    args.nreps = args.nreps / 2 * 2 + 1;

    // Validate parameters
    if args.nprices < 2
        || args.lookback < 2
        || args.lookahead < 1
        || args.ntrain < 2
        || args.ntest < 1
    {
        eprintln!("Error: Invalid parameters");
        std::process::exit(1);
    }

    if args.nprices < args.lookback + args.lookahead + args.ntrain + args.ntest + 10 {
        eprintln!(
            "Error: nprices must be at least lookback + lookahead + ntrain + ntest + 10"
        );
        std::process::exit(1);
    }

    println!(
        "\nnprices={}  lookback={}  lookahead={}  ntrain={}  ntest={}  omit={}  extra={}",
        args.nprices, args.lookback, args.lookahead, args.ntrain, args.ntest, args.omit, args.extra
    );

    let mut rng = Mwc256::new(123456789);
    let mut save_t = vec![0.0; args.nreps];
    let mut p1_count = 0;

    for irep in 0..args.nreps {
        // Generate random walk prices
        let mut x = vec![0.0; args.nprices];
        for i in 1..args.nprices {
            x[i] = x[i - 1] + rng.unifrand() + rng.unifrand() - rng.unifrand() - rng.unifrand();
        }

        // Build dataset of indicators and targets
        let mut data = Vec::new();
        for i in 0..(args.nprices - args.lookback - args.lookahead + 1) {
            let (ind, targ) = ind_targ(args.lookback, args.lookahead, &x[i..]);
            data.push((ind, targ));
        }

        let ncases = data.len();

        // Perform walkforward validation
        let mut oos = Vec::new();
        let mut trn_start = 0;
        let mut istart = args.ntrain;

        loop {
            let test_start = trn_start + args.ntrain;
            if test_start >= ncases {
                break;
            }

            // Train on ntrain - omit cases
            let train_data = &data[trn_start..(trn_start + args.ntrain - args.omit)];
            let (beta, constant) = find_beta(train_data);

            // Test on ntest cases (or fewer if at end)
            let mut nt = args.ntest;
            if nt > ncases - istart {
                nt = ncases - istart;
            }

            for itest in 0..nt {
                let test_idx = test_start + itest;
                if test_idx >= ncases {
                    break;
                }
                let (ind, targ) = data[test_idx];
                let pred = beta * ind + constant;

                if pred > 0.0 {
                    oos.push(targ);
                } else {
                    oos.push(-targ);
                }
            }

            istart += nt + args.extra;
            trn_start += nt + args.extra;
        }

        // Analyze results
        let n_oos = oos.len();
        let oos_mean: f64 = oos.iter().sum::<f64>() / (n_oos as f64);
        let oos_ss: f64 = oos.iter().map(|&x| x * x).sum::<f64>() / (n_oos as f64);
        let oos_var = (oos_ss - oos_mean * oos_mean).max(1e-20);

        let t = (n_oos as f64).sqrt() * oos_mean / oos_var.sqrt();
        let rtail = 1.0 - normal_cdf(t);

        println!(
            "Mean = {:.4}  StdDev = {:.4}  t = {:.4}  p = {:.4}",
            oos_mean,
            oos_var.sqrt(),
            t,
            rtail
        );

        save_t[irep] = t;

        if rtail <= 0.1 {
            p1_count += 1;
        }
    }

    // Sort and report median
    quicksort(&mut save_t);
    let n_oos = {
        // Recalculate n_oos for the last replication (they should all be the same)
        let mut x = vec![0.0; args.nprices];
        for i in 1..args.nprices {
            x[i] = x[i - 1] + rng.unifrand() + rng.unifrand() - rng.unifrand() - rng.unifrand();
        }
        let mut data = Vec::new();
        for i in 0..(args.nprices - args.lookback - args.lookahead + 1) {
            let (ind, targ) = ind_targ(args.lookback, args.lookahead, &x[i..]);
            data.push((ind, targ));
        }
        let ncases = data.len();
        let mut count = 0;
        let mut istart = args.ntrain;
        let mut trn_start = 0;
        loop {
            let test_start = trn_start + args.ntrain;
            if test_start >= ncases {
                break;
            }
            let mut nt = args.ntest;
            if nt > ncases - istart {
                nt = ncases - istart;
            }
            count += nt;
            istart += nt + args.extra;
            trn_start += nt + args.extra;
        }
        count
    };

    println!(
        "\nn OOS = {}  Median t = {:.4}  Fraction with p<= 0.1 = {:.3}",
        n_oos,
        save_t[args.nreps / 2],
        (p1_count as f64) / (args.nreps as f64)
    );
}
