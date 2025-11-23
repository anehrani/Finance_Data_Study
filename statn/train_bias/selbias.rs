// Selection bias analysis module

use crate::rng::Rng;
use crate::opt::{OptCriteria, opt_params};
use crate::test_system::test_system;

pub fn run_selection_bias(
    criteria: OptCriteria,
    ncases: usize,
    save_trend: f64,
    nreps: usize,
) {
    println!(
        "\n\nwhich={:?} ncases={} trend={:.3} nreps={}",
        criteria, ncases, save_trend, nreps
    );

    // Initialize RNG
    let mut rng = Rng::with_seed(123456789);

    let mut l_is_mean = 0.0;
    let mut s_is_mean = 0.0;
    let mut l_oos_mean = 0.0;
    let mut s_oos_mean = 0.0;
    let mut oos_mean = 0.0;
    let mut bias_mean = 0.0;
    let mut bias_ss = 0.0;

    // Main replication loop
    for irep in 0..nreps {
        // Generate in-sample set (log prices)
        let mut x = vec![0.0; ncases];
        let mut trend = save_trend;
        x[0] = 0.0;

        for i in 1..ncases {
            if (i + 1) % 50 == 0 {
                trend = -trend;
            }
            x[i] = x[i - 1]
                + trend
                + rng.unifrand()
                + rng.unifrand()
                - rng.unifrand()
                - rng.unifrand();
        }

        // Compute optimal parameters for long-only model
        let (_l_best_perf, l_short_lookback, l_long_lookback) = opt_params(criteria, true, &x);
        let l_is_perf = test_system(true, &x, l_short_lookback, l_long_lookback);

        // Compute optimal parameters for short-only model
        let (_s_best_perf, s_short_lookback, s_long_lookback) = opt_params(criteria, false, &x);
        let s_is_perf = test_system(false, &x, s_short_lookback, s_long_lookback);

        // Generate first out-of-sample set (log prices)
        // This will give us the performance results on which our choice of model is based
        let mut x_oos1 = vec![0.0; ncases];
        trend = save_trend;
        x_oos1[0] = 0.0;

        for i in 1..ncases {
            if (i + 1) % 50 == 0 {
                trend = -trend;
            }
            x_oos1[i] = x_oos1[i - 1]
                + trend
                + rng.unifrand()
                + rng.unifrand()
                - rng.unifrand()
                - rng.unifrand();
        }

        // Test first OOS set with both models
        let l_oos_perf = test_system(true, &x_oos1, l_short_lookback, l_long_lookback);
        let s_oos_perf = test_system(false, &x_oos1, s_short_lookback, s_long_lookback);

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

        // Generate second out-of-sample set (log prices)
        // This is the 'ultimate' OOS set, which has selection bias removed
        let mut x_oos2 = vec![0.0; ncases];
        trend = save_trend;
        x_oos2[0] = 0.0;

        for i in 1..ncases {
            if (i + 1) % 50 == 0 {
                trend = -trend;
            }
            x_oos2[i] = x_oos2[i - 1]
                + trend
                + rng.unifrand()
                + rng.unifrand()
                - rng.unifrand()
                - rng.unifrand();
        }

        // Choose either the long or the short model, depending on which
        // did better on the first OOS set
        let (oos_perf, bias) = if l_oos_perf > s_oos_perf {
            let oos = test_system(true, &x_oos2, l_short_lookback, l_long_lookback);
            (oos, l_oos_perf - oos)
        } else {
            let oos = test_system(false, &x_oos2, s_short_lookback, s_long_lookback);
            (oos, s_oos_perf - oos)
        };

        bias_mean += bias;
        bias_ss += bias * bias;
        oos_mean += oos_perf;
        println!("     OOS_perf={:8.4}  Bias={:8.4}", oos_perf, bias);
    }

    // Print final results
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
    if bias_ss < 1.0e-20 {
        bias_ss = 1.0e-20;
    }
    let t = (nreps as f64).sqrt() * bias_mean / bias_ss.sqrt();

    println!(
        "OOS={:.4}  Selection bias={:.4}  t={:.3}",
        oos_mean, bias_mean, t
    );

    println!("\nCompleted...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}
