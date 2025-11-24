// Training bias analysis module

use crate::rng::Rng;
use crate::opt::{OptCriteria, opt_params_both_directions};
use crate::test_system::test_system_both_directions;

pub fn run_training_bias(
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
                + rng.unifrand()
                + rng.unifrand()
                - rng.unifrand()
                - rng.unifrand();
        }

        // Compute optimal parameters, evaluate return with same dataset
        // For training bias, we trade both long and short positions (original behavior)
        let (_best_perf, short_lookback, long_lookback) = opt_params_both_directions(criteria, &x);
        let is_perf = test_system_both_directions(&x, short_lookback, long_lookback);

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
                + rng.unifrand()
                + rng.unifrand()
                - rng.unifrand()
                - rng.unifrand();
        }

        // Test the OOS set
        let oos_perf = test_system_both_directions(&x_oos, short_lookback, long_lookback);

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

    println!("\nCompleted...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}
