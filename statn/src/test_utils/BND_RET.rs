use std::fs::File;
use std::io::{BufRead, BufReader};

mod qsorts;
mod stats;

use qsorts::qsortd;
use stats::{orderstat_tail, quantile_conf};

const MKTBUF: usize = 2048;

/// Compute optimal short-term and long-term lookbacks for a moving-average crossover system
fn opt_params(
    ncases: usize,
    max_lookback: usize,
    x: &[f64],
    short_term: &mut usize,
    long_term: &mut usize,
) -> f64 {
    let mut best_perf = f64::NEG_INFINITY;
    let mut ibestshort = 1;
    let mut ibestlong = 2;

    for ilong in 2..max_lookback {
        for ishort in 1..ilong {
            let mut total_return = 0.0;

            let mut short_sum = 0.0;
            let mut long_sum = 0.0;

            for i in (ilong - 1)..(ncases - 1) {
                if i == ilong - 1 {
                    // Compute initial short-term and long-term moving averages
                    short_sum = 0.0;
                    for j in (i.saturating_sub(ishort - 1))..=i {
                        short_sum += x[j];
                    }

                    long_sum = short_sum;
                    let mut j = i as i32 - ishort as i32;
                    while j >= (i as i32 - ilong as i32) {
                        if j >= 0 {
                            long_sum += x[j as usize];
                        }
                        j -= 1;
                    }
                } else {
                    // Update the moving averages
                    short_sum += x[i] - x[i - ishort];
                    long_sum += x[i] - x[i - ilong];
                }

                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;

                // Compute position and return
                let ret = if short_mean > long_mean {
                    // Long position
                    x[i + 1] - x[i]
                } else if short_mean < long_mean {
                    // Short position
                    x[i] - x[i + 1]
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

    *short_term = ibestshort;
    *long_term = ibestlong;

    best_perf
}

/// Test a trained crossover system and return mean return
fn test_system(ncases: usize, x: &[f64], short_term: usize, long_term: usize) -> f64 {
    let mut sum = 0.0;
    let mut n = ncases;
    let mut i = long_term - 1;

    while n > 0 && i + 1 < x.len() {
        let mut short_sum = 0.0;
        for j in (i.saturating_sub(short_term - 1))..=i {
            short_sum += x[j];
        }

        let mut long_sum = short_sum;
        let mut j = i as i32 - short_term as i32;
        while j >= (i as i32 - long_term as i32) {
            if j >= 0 {
                long_sum += x[j as usize];
            }
            j -= 1;
        }

        let short_mean = short_sum / short_term as f64;
        let long_mean = long_sum / long_term as f64;

        // Compute position and return
        if short_mean > long_mean {
            sum += x[i + 1] - x[i];
        } else if short_mean < long_mean {
            sum -= x[i + 1] - x[i];
        }

        n -= 1;
        i += 1;
    }

    sum / ncases as f64
}

/// Read market prices from a file
fn read_market_prices(filename: &str) -> Result<Vec<f64>, String> {
    let file = File::open(filename).map_err(|e| format!("Cannot open file: {}", e))?;
    let reader = BufReader::new(file);

    let mut prices = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Error reading line {}: {}", line_num + 1, e))?;

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse date (YYYYMMDD) - validate first 8 characters are digits
        if line.len() < 9 {
            return Err(format!("Invalid date on line {}", line_num + 1));
        }

        for i in 0..8 {
            if !line.chars().nth(i).unwrap().is_ascii_digit() {
                return Err(format!("Invalid date reading line {}", line_num + 1));
            }
        }

        // Parse price (skip column 9 and handle delimiters)
        let price_part = &line[9..];
        let price_str = price_part
            .trim_start_matches(|c: char| c == ' ' || c == '\t' || c == ',')
            .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
            .next()
            .unwrap_or("0");

        match price_str.parse::<f64>() {
            Ok(price) if price > 0.0 => {
                prices.push(price.ln()); // Store log price
            }
            Ok(_) => {
                return Err(format!("Invalid price on line {}: {}", line_num + 1, price_str));
            }
            Err(e) => {
                return Err(format!("Cannot parse price on line {}: {}", line_num + 1, e));
            }
        }
    }

    if prices.is_empty() {
        return Err("No prices read from file".to_string());
    }

    Ok(prices)
}

fn main() {
    // Command line arguments
    let args: Vec<String> = std::env::args().collect();

    let (max_lookback, n_train, n_test, lower_fail_rate, upper_fail_rate, p_of_q, filename) =
        if args.len() == 8 {
            let max_lookback = args[1].parse::<usize>().expect("Invalid max_lookback");
            let n_train = args[2].parse::<usize>().expect("Invalid n_train");
            let n_test = args[3].parse::<usize>().expect("Invalid n_test");
            let lower_fail_rate = args[4].parse::<f64>().expect("Invalid lower_fail_rate");
            let upper_fail_rate = args[5].parse::<f64>().expect("Invalid upper_fail_rate");
            let p_of_q = args[6].parse::<f64>().expect("Invalid p_of_q");
            let filename = args[7].clone();

            (
                max_lookback,
                n_train,
                n_test,
                lower_fail_rate,
                upper_fail_rate,
                p_of_q,
                filename,
            )
        } else {
            eprintln!("Usage: bnd_ret <max_lookback> <n_train> <n_test> <lower_fail> <upper_fail> <p_of_q> <filename>");
            eprintln!("  max_lookback - Maximum moving-average lookback");
            eprintln!("  n_train - Number of bars in training set (much greater than max_lookback)");
            eprintln!("  n_test - Number of bars in test set");
            eprintln!("  lower_fail - Lower bound failure rate (often 0.01-0.1)");
            eprintln!("  upper_fail - Upper bound failure rate (often 0.1-0.5)");
            eprintln!("  p_of_q - Probability of bad bound (often 0.01-0.1)");
            eprintln!("  filename - name of market file (YYYYMMDD Price)");
            std::process::exit(1);
        };

    if n_train - max_lookback < 10 {
        eprintln!("ERROR... n_train must be at least 10 greater than max_lookback");
        std::process::exit(1);
    }

    // Read market prices
    println!("Reading market file...");
    let prices = match read_market_prices(&filename) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error reading market file: {}", e);
            std::process::exit(1);
        }
    };

    let nprices = prices.len();
    println!("Market price history read");

    if n_train + n_test > nprices {
        eprintln!(
            "ERROR... n_train + n_test must not exceed n_prices ({})",
            nprices
        );
        std::process::exit(1);
    }

    let mut returns = Vec::new();
    let mut total = 0.0;
    let mut train_start = 0;

    // Walkforward analysis
    loop {
        let mut short_lookback = 0;
        let mut long_lookback = 0;

        let is_perf = opt_params(
            n_train,
            max_lookback,
            &prices[train_start..train_start + n_train],
            &mut short_lookback,
            &mut long_lookback,
        );

        let is = is_perf * 25200.0; // Annualize
        println!(
            "\nIS = {:.3} at {}  Lookback={} {}",
            is, train_start, short_lookback, long_lookback
        );

        let mut n = n_test;
        if n > nprices - train_start - n_train {
            n = nprices - train_start - n_train;
        }

        let test_start = train_start + n_train - long_lookback;
        let oos_perf = test_system(n, &prices[test_start..], short_lookback, long_lookback);
        let oos = oos_perf * 25200.0; // Annualize
        println!("\nOOS = {:.3} at {}", oos, train_start + n_train);

        returns.push(oos);
        total += oos;

        // Advance fold window
        train_start += n;
        if train_start + n_train >= nprices {
            break;
        }
    }

    let n_returns = returns.len();
    println!(
        "\n\nAll returns are approximately annualized by multiplying by 25200"
    );
    println!(
        "mean OOS = {:.3} with {} returns",
        total / n_returns as f64,
        n_returns
    );

    // Return bounding
    qsortd(0, (returns.len() as i32) - 1, &mut returns);

    let lower_bound_m =
        ((lower_fail_rate * (n_returns as f64 + 1.0)) as usize).max(1);
    let lower_bound = returns[lower_bound_m - 1];

    let upper_bound_m =
        ((upper_fail_rate * (n_returns as f64 + 1.0)) as usize).max(1);
    let upper_bound = returns[n_returns - upper_bound_m];

    let lower_bound_opt_q = 0.9 * lower_fail_rate;
    let lower_bound_pes_q = 1.1 * lower_fail_rate;

    let upper_bound_opt_q = 0.9 * upper_fail_rate;
    let upper_bound_pes_q = 1.1 * upper_fail_rate;

    let lower_bound_opt_prob =
        1.0 - orderstat_tail(n_returns as i32, lower_bound_opt_q, lower_bound_m as i32);
    let lower_bound_pes_prob =
        orderstat_tail(n_returns as i32, lower_bound_pes_q, lower_bound_m as i32);

    let upper_bound_opt_prob =
        1.0 - orderstat_tail(n_returns as i32, upper_bound_opt_q, upper_bound_m as i32);
    let upper_bound_pes_prob =
        orderstat_tail(n_returns as i32, upper_bound_pes_q, upper_bound_m as i32);

    let lower_bound_p_of_q_opt_q =
        quantile_conf(n_returns as i32, lower_bound_m as i32, 1.0 - p_of_q);
    let lower_bound_p_of_q_pes_q =
        quantile_conf(n_returns as i32, lower_bound_m as i32, p_of_q);

    let upper_bound_p_of_q_opt_q =
        quantile_conf(n_returns as i32, upper_bound_m as i32, 1.0 - p_of_q);
    let upper_bound_p_of_q_pes_q =
        quantile_conf(n_returns as i32, upper_bound_m as i32, p_of_q);

    // Output results
    println!("\n\nThe LOWER bound on future returns is {:.3}", lower_bound);
    println!(
        "It has an expected user-specified failure rate of {:.2} %",
        100.0 * lower_fail_rate
    );
    println!("  (This is the percent of future returns less than the lower bound.)");

    println!(
        "\n\nWe may take an optimistic view: the lower bound is too low."
    );
    println!("  (This results in a lower failure rate.)");
    println!(
        "The probability is {:.4} that the true failure rate is {:.2} % or less",
        lower_bound_opt_prob, 100.0 * lower_bound_opt_q
    );
    println!(
        "The probability is {:.4} that the true failure rate is {:.2} % or less",
        p_of_q, 100.0 * lower_bound_p_of_q_opt_q
    );

    println!(
        "\n\nWe may take a pessimistic view: the lower bound is too high."
    );
    println!("  (This results in a higher failure rate.)");
    println!(
        "The probability is {:.4} that the true failure rate is {:.2} % or more",
        lower_bound_pes_prob, 100.0 * lower_bound_pes_q
    );
    println!(
        "The probability is {:.4} that the true failure rate is {:.2} % or more",
        p_of_q, 100.0 * lower_bound_p_of_q_pes_q
    );

    println!("\n\nThe UPPER bound on future returns is {:.3}", upper_bound);
    println!(
        "It has an expected user-specified failure rate of {:.2} %",
        100.0 * upper_fail_rate
    );
    println!("  (This is the percent of future returns greater than the upper bound.)");

    println!(
        "\n\nWe may take an optimistic view: the upper bound is too high."
    );
    println!("  (This results in a lower failure rate.)");
    println!(
        "The probability is {:.4} that the true failure rate is {:.2} % or less",
        upper_bound_opt_prob, 100.0 * upper_bound_opt_q
    );
    println!(
        "The probability is {:.4} that the true failure rate is {:.2} % or less",
        p_of_q, 100.0 * upper_bound_p_of_q_opt_q
    );

    println!(
        "\n\nWe may take a pessimistic view: the upper bound is too low."
    );
    println!("  (This results in a higher failure rate.)");
    println!(
        "The probability is {:.4} that the true failure rate is {:.2} % or more",
        upper_bound_pes_prob, 100.0 * upper_bound_pes_q
    );
    println!(
        "The probability is {:.4} that the true failure rate is {:.2} % or more",
        p_of_q, 100.0 * upper_bound_p_of_q_pes_q
    );

    println!("\nAnalysis complete.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_params() {
        let log_prices = vec![
            0.0, 0.01, 0.02, 0.015, 0.025, 0.03, 0.028, 0.035, 0.04, 0.038,
        ];
        let mut short_term = 0;
        let mut long_term = 0;
        let perf = opt_params(log_prices.len(), 5, &log_prices, &mut short_term, &mut long_term);
        assert!(perf.is_finite());
        assert!(short_term < long_term);
    }

    #[test]
    fn test_test_system() {
        let log_prices = vec![
            0.0, 0.01, 0.02, 0.015, 0.025, 0.03, 0.028, 0.035, 0.04, 0.038,
        ];
        let mean_return = test_system(log_prices.len() - 3, &log_prices, 2, 4);
        assert!(mean_return.is_finite());
    }
}