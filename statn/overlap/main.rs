use clap::Parser;
use stats::normal_cdf;

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

use matlib::{Mwc256, qsortd, ind_targ, find_beta};

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

    let mut rng = Mwc256::with_seed(123456789);
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
            let (ind, targ) = ind_targ(args.lookback, args.lookahead, &x, i + args.lookback - 1);
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
    if !save_t.is_empty() {
        qsortd(0, save_t.len() - 1, &mut save_t);
    }
    let n_oos = {
        // Recalculate n_oos for the last replication (they should all be the same)
        let mut x = vec![0.0; args.nprices];
        for i in 1..args.nprices {
            x[i] = x[i - 1] + rng.unifrand() + rng.unifrand() - rng.unifrand() - rng.unifrand();
        }
        let mut data = Vec::new();
        for i in 0..(args.nprices - args.lookback - args.lookahead + 1) {
            let (ind, targ) = ind_targ(args.lookback, args.lookahead, &x, i + args.lookback - 1);
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
