mod stats;

use stats::{orderstat_tail, quantile_conf};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

/// Compute optimal short-term and long-term lookbacks
/// for a primitive moving-average crossover system
fn opt_params(
    ncases: usize,
    max_lookback: usize,
    x: &[f64],
) -> (f64, usize, usize) {
    let mut best_perf = -1e60;
    let mut ibestshort = 0;
    let mut ibestlong = 0;

    for ilong in 2..max_lookback {
        for ishort in 1..ilong {
            let mut total_return = 0.0;

            let mut short_sum = 0.0;
            let mut long_sum = 0.0;

            for i in (ilong - 1)..(ncases - 1) {
                if i == ilong - 1 {
                    // Initialize sums for first valid case
                    short_sum = 0.0;
                    for j in (i - ishort + 1)..=i {
                        short_sum += x[j];
                    }
                    long_sum = short_sum;
                    for j in (i - ilong + 1)..=(i - ishort) {
                        long_sum += x[j];
                    }
                } else {
                    // Update moving averages
                    short_sum += x[i] - x[i - ishort];
                    long_sum += x[i] - x[i - ilong];
                }

                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;

                // Take position and cumulate performance
                let ret = if short_mean > long_mean {
                    x[i + 1] - x[i] // Long position
                } else if short_mean < long_mean {
                    x[i] - x[i + 1] // Short position
                } else {
                    0.0
                };

                total_return += ret;
            }

            total_return /= (ncases - ilong) as f64;
            if total_return > best_perf {
                best_perf = total_return;
                ibestshort = ishort;
                ibestlong = ilong;
            }
        }
    }

    (best_perf, ibestshort, ibestlong)
}

/// Test a trained crossover system
/// This computes the mean return
fn test_system(
    ncases: usize,
    x: &[f64],
    short_term: usize,
    long_term: usize,
) -> f64 {
    let mut sum = 0.0;
    let mut n = ncases;

    let mut i = long_term - 1;
    while n > 0 {
        let mut short_mean = 0.0;
        for j in (i - short_term + 1)..=i {
            short_mean += x[j];
        }

        let mut long_mean = short_mean;
        for j in (i - long_term + 1)..=(i - short_term) {
            long_mean += x[j];
        }

        short_mean /= short_term as f64;
        long_mean /= long_term as f64;

        // Take position and cumulate return
        if short_mean > long_mean {
            sum += x[i + 1] - x[i]; // Long position
        } else if short_mean < long_mean {
            sum -= x[i + 1] - x[i]; // Short position
        }

        n -= 1;
        i += 1;
    }

    sum / ncases as f64
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 8 {
        eprintln!("\nUsage: {} max_lookback n_train n_test lower_fail upper_fail p_of_q filename", args[0]);
        eprintln!("  max_lookback - Maximum moving-average lookback");
        eprintln!("  n_train - Number of bars in training set (much greater than max_lookback)");
        eprintln!("  n_test - Number of bars in test set");
        eprintln!("  lower_fail - Lower bound failure rate (often 0.01-0.1)");
        eprintln!("  upper_fail - Upper bound failure rate (often 0.1-0.5)");
        eprintln!("  p_of_q - Probability of bad bound (often 0.01-0.1)");
        eprintln!("  filename - name of market file (YYYYMMDD Price)");
        process::exit(1);
    }

    let max_lookback: usize = args[1].parse().expect("Invalid max_lookback");
    let n_train: usize = args[2].parse().expect("Invalid n_train");
    let n_test: usize = args[3].parse().expect("Invalid n_test");
    let lower_fail_rate: f64 = args[4].parse().expect("Invalid lower_fail");
    let upper_fail_rate: f64 = args[5].parse().expect("Invalid upper_fail");
    let p_of_q: f64 = args[6].parse().expect("Invalid p_of_q");
    let filename = &args[7];

    if n_train - max_lookback < 10 {
        eprintln!("\nERROR... n_train must be at least 10 greater than max_lookback");
        process::exit(1);
    }

    // Read market prices
    println!("\nReading market file...");

    let file = File::open(filename).unwrap_or_else(|_| {
        eprintln!("\n\nCannot open market history file {}", filename);
        process::exit(1);
    });

    let reader = BufReader::new(file);
    let mut prices = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap_or_else(|_| {
            eprintln!("\nError reading file");
            process::exit(1);
        });

        if line.len() < 2 {
            break;
        }

        // Parse date (crude sanity check)
        if line.len() < 8 || !line[..8].chars().all(|c| c.is_ascii_digit()) {
            eprintln!("\nInvalid date in line: {}", line);
            process::exit(1);
        }

        // Parse price
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            eprintln!("\nInvalid line format: {}", line);
            process::exit(1);
        }

        let price: f64 = parts[1].parse().unwrap_or_else(|_| {
            eprintln!("\nInvalid price in line: {}", line);
            process::exit(1);
        });

        if price > 0.0 {
            prices.push(price.ln());
        }
    }

    let nprices = prices.len();
    println!("\nMarket price history read");

    if n_train + n_test > nprices {
        eprintln!("\nERROR... n_train + n_test must not exceed n_prices.");
        process::exit(1);
    }

    let mut returns = Vec::new();
    let mut train_start = 0;
    let mut total = 0.0;

    // Do walkforward
    loop {
        let (is_perf, short_lookback, long_lookback) = opt_params(
            n_train,
            max_lookback,
            &prices[train_start..train_start + n_train],
        );
        let is_annualized = is_perf * 25200.0;
        println!(
            "\n\nIS = {:.3} at {}  Lookback={} {}",
            is_annualized, train_start, short_lookback, long_lookback
        );

        let mut n = n_test;
        if n > nprices - train_start - n_train {
            n = nprices - train_start - n_train;
        }

        let oos = test_system(
            n,
            &prices[train_start + n_train - long_lookback..],
            short_lookback,
            long_lookback,
        );
        let oos_annualized = oos * 25200.0;
        println!("OOS = {:.3} at {}", oos_annualized, train_start + n_train);

        returns.push(oos_annualized);
        total += oos_annualized;

        train_start += n;
        if train_start + n_train >= nprices {
            break;
        }
    }

    let n_returns = returns.len();
    println!("\n\nAll returns are approximately annualized by multiplying by 25200");
    println!("mean OOS = {:.3} with {} returns", total / n_returns as f64, n_returns);

    // Do return bounding
    returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let lower_bound_m = ((lower_fail_rate * (n_returns as f64 + 1.0)) as usize).max(1);
    let lower_bound = returns[lower_bound_m - 1];

    let upper_bound_m = ((upper_fail_rate * (n_returns as f64 + 1.0)) as usize).max(1);
    let upper_bound = returns[n_returns - upper_bound_m];

    let lower_bound_opt_q = 0.9 * lower_fail_rate;
    let lower_bound_pes_q = 1.1 * lower_fail_rate;

    let upper_bound_opt_q = 0.9 * upper_fail_rate;
    let upper_bound_pes_q = 1.1 * upper_fail_rate;

    let lower_bound_opt_prob = 1.0 - orderstat_tail(n_returns as i32, lower_bound_opt_q, lower_bound_m as i32);
    let lower_bound_pes_prob = orderstat_tail(n_returns as i32, lower_bound_pes_q, lower_bound_m as i32);

    let upper_bound_opt_prob = 1.0 - orderstat_tail(n_returns as i32, upper_bound_opt_q, upper_bound_m as i32);
    let upper_bound_pes_prob = orderstat_tail(n_returns as i32, upper_bound_pes_q, upper_bound_m as i32);

    let lower_bound_p_of_q_opt_q = quantile_conf(n_returns as i32, lower_bound_m as i32, 1.0 - p_of_q);
    let lower_bound_p_of_q_pes_q = quantile_conf(n_returns as i32, lower_bound_m as i32, p_of_q);

    let upper_bound_p_of_q_opt_q = quantile_conf(n_returns as i32, upper_bound_m as i32, 1.0 - p_of_q);
    let upper_bound_p_of_q_pes_q = quantile_conf(n_returns as i32, upper_bound_m as i32, p_of_q);

    println!("\n\nThe LOWER bound on future returns is {:.3}", lower_bound);
    println!("It has an expected user-specified failure rate of {:.2} %", 100.0 * lower_fail_rate);
    println!("  (This is the percent of future returns less than the lower bound.)");

    println!("\n\nWe may take an optimistic view: the lower bound is too low.");
    println!("  (This results in a lower failure rate.)");
    println!("The probability is {:.4} that the true failure rate is {:.2} % or less",
             lower_bound_opt_prob, 100.0 * lower_bound_opt_q);
    println!("The probability is {:.4} that the true failure rate is {:.2} % or less",
             p_of_q, 100.0 * lower_bound_p_of_q_opt_q);

    println!("\n\nWe may take a pessimistic view: the lower bound is too high.");
    println!("  (This results in a higher failure rate.)");
    println!("The probability is {:.4} that the true failure rate is {:.2} % or more",
             lower_bound_pes_prob, 100.0 * lower_bound_pes_q);
    println!("The probability is {:.4} that the true failure rate is {:.2} % or more",
             p_of_q, 100.0 * lower_bound_p_of_q_pes_q);

    println!("\n\nThe UPPER bound on future returns is {:.3}", upper_bound);
    println!("It has an expected user-specified failure rate of {:.2} %", 100.0 * upper_fail_rate);
    println!("  (This is the percent of future returns greater than the upper bound.)");

    println!("\n\nWe may take an optimistic view: the upper bound is too high.");
    println!("  (This results in a lower failure rate.)");
    println!("The probability is {:.4} that the true failure rate is {:.2} % or less",
             upper_bound_opt_prob, 100.0 * upper_bound_opt_q);
    println!("The probability is {:.4} that the true failure rate is {:.2} % or less",
             p_of_q, 100.0 * upper_bound_p_of_q_opt_q);

    println!("\n\nWe may take a pessimistic view: the upper bound is too low.");
    println!("  (This results in a higher failure rate.)");
    println!("The probability is {:.4} that the true failure rate is {:.2} % or more",
             upper_bound_pes_prob, 100.0 * upper_bound_pes_q);
    println!("The probability is {:.4} that the true failure rate is {:.2} % or more",
             p_of_q, 100.0 * upper_bound_p_of_q_pes_q);
}
