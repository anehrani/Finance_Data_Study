use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

const MKTBUF: usize = 2048;

// External functions already implemented in Rust
fn t_cdf(ndf: i32, t: f64) -> f64;
fn inverse_t_cdf(ndf: i32, p: f64) -> f64;

// Bootstrap functions (implementations not shown, assumed available)
fn boot_conf_pctile<F>(
    x: &[f64],
    user_t: F,
    nboot: usize,
) -> (f64, f64, f64, f64, f64, f64)
where
    F: Fn(&[f64]) -> f64,
{
    // Implementation would go here
    (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
}

fn boot_conf_bca<F>(
    x: &[f64],
    user_t: F,
    nboot: usize,
) -> (f64, f64, f64, f64, f64, f64)
where
    F: Fn(&[f64]) -> f64,
{
    // Implementation would go here
    (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
}

/// Computes optimal lookback and breakout threshold for a primitive long-only moving-average breakout system.
fn opt_params(
    prices: &[f64],
    max_lookback: usize,
) -> (f64, usize, f64, usize) {
    let nprices = prices.len();
    let mut best_perf = -1.0e60;
    let mut ibestlook = 2;
    let mut ibestthresh = 1;
    let mut last_position_of_best = 0;

    for ilook in 2..=max_lookback {
        for ithresh in 1..=10 {
            let mut total_return = 0.0;
            let mut n_trades = 0;
            let mut position = 0;
            let mut ma_sum = 0.0;

            for i in (max_lookback - 1)..(nprices - 1) {
                if i == max_lookback - 1 {
                    ma_sum = 0.0;
                    for j in (i - ilook + 1)..=i {
                        ma_sum += prices[j];
                    }
                } else {
                    ma_sum += prices[i] - prices[i - ilook];
                }

                let ma_mean = ma_sum / ilook as f64;
                let trial_thresh = 1.0 + 0.01 * ithresh as f64;

                if prices[i] > trial_thresh * ma_mean {
                    position = 1;
                } else if prices[i] < ma_mean {
                    position = 0;
                }

                let ret = if position != 0 {
                    prices[i + 1] - prices[i]
                } else {
                    0.0
                };

                if position != 0 {
                    n_trades += 1;
                    total_return += ret;
                }
            }

            total_return /= (n_trades as f64) + 1.0e-30;
            if total_return > best_perf {
                best_perf = total_return;
                ibestlook = ilook;
                ibestthresh = ithresh;
                last_position_of_best = position;
            }
        }
    }

    (best_perf, ibestlook, 0.01 * ibestthresh as f64, last_position_of_best)
}

/// Computes return vector for all bars according to user's request.
fn comp_return(
    ret_type: i32,
    prices: &[f64],
    istart: usize,
    ntest: usize,
    lookback: usize,
    thresh: f64,
    last_pos: usize,
) -> Vec<f64> {
    let nprices = prices.len();
    let mut returns = Vec::new();
    let mut position = last_pos;
    let mut prior_position = 0;
    let trial_thresh = 1.0 + thresh;
    let mut ma_sum = 0.0;

    for i in (istart - 1)..(istart - 1 + ntest) {
        if i == istart - 1 {
            ma_sum = 0.0;
            for j in (i - lookback + 1)..=i {
                if j >= 0 {
                    ma_sum += prices[j];
                }
            }
        } else if i >= lookback {
            ma_sum += prices[i] - prices[i - lookback];
        }

        let ma_mean = ma_sum / lookback as f64;

        if i + 1 < nprices {
            if prices[i] > trial_thresh * ma_mean {
                position = 1;
            } else if prices[i] < ma_mean {
                position = 0;
            }

            let ret = if position != 0 {
                prices[i + 1] - prices[i]
            } else {
                0.0
            };

            match ret_type {
                0 => {
                    // All bars
                    returns.push(ret);
                }
                1 => {
                    // Only bars with a position
                    if position != 0 {
                        returns.push(ret);
                    }
                }
                2 => {
                    // Completed trades
                    if position != 0 && prior_position == 0 {
                        // Just opened a trade
                        // Store open price implicitly
                    } else if prior_position != 0 && position == 0 {
                        // Just closed a trade
                        returns.push(prices[i] - prices[i]);
                    } else if position != 0 && i == istart - 2 + ntest {
                        // Force close at end of data
                        if i + 1 < nprices {
                            returns.push(prices[i + 1] - prices[i]);
                        }
                    }
                }
                _ => {}
            }

            prior_position = position;
        }
    }

    returns
}

/// Computes the mean of a slice
fn find_mean(x: &[f64]) -> f64 {
    if x.is_empty() {
        return 0.0;
    }
    x.iter().sum::<f64>() / x.len() as f64
}

/// Reads market prices from file
fn read_market_file(filename: &str) -> io::Result<Vec<f64>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut prices = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() || line.len() < 2 {
            continue;
        }

        // Parse date (first 8 characters should be digits)
        if line.len() < 9 {
            continue;
        }

        let date_part = &line[0..8];
        if !date_part.chars().all(|c| c.is_ascii_digit()) {
            eprintln!("Invalid date in line: {}", line);
            continue;
        }

        // Parse price (from position 9 onwards, skipping whitespace/delimiters)
        let price_part = &line[9..];
        let price_str = price_part.trim_start_matches(|c: char| c.is_whitespace() || c == ',' || c == '\t');

        if let Ok(price) = price_str.parse::<f64>() {
            if price > 0.0 {
                prices.push(price.ln());
            }
        }
    }

    Ok(prices)
}

/// Computes statistics for a return series
fn compute_stats(returns: &[f64]) -> (f64, f64, f64, f64, f64) {
    if returns.is_empty() {
        return (0.0, 0.0, 0.0, 1.0, 0.0);
    }

    let n = returns.len() as f64;
    let mean = returns.iter().sum::<f64>() / n;

    if returns.len() < 2 {
        return (mean, 0.0, 0.0, 1.0, 0.0);
    }

    let variance = returns.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let stddev = variance.sqrt();
    let t = (n.sqrt() * mean) / (stddev + 1.0e-20);
    let p = 1.0 - t_cdf((returns.len() - 1) as i32, t);
    let t_lower = mean - (stddev / n.sqrt()) * inverse_t_cdf((returns.len() - 1) as i32, 0.9);

    (mean, stddev, t, p, t_lower)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 6 {
        eprintln!("Usage: bound_mean max_lookback n_train n_test n_boot filename");
        eprintln!("  max_lookback - Maximum moving-average lookback");
        eprintln!("  n_train - Number of bars in training set");
        eprintln!("  n_test - Number of bars in test set");
        eprintln!("  n_boot - Number of bootstrap reps");
        eprintln!("  filename - name of market file (YYYYMMDD Price)");
        return;
    }

    let max_lookback: usize = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid max_lookback");
            return;
        }
    };

    let n_train: usize = match args[2].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid n_train");
            return;
        }
    };

    let n_test: usize = match args[3].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid n_test");
            return;
        }
    };

    let n_boot: usize = match args[4].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Invalid n_boot");
            return;
        }
    };

    let filename = &args[5];

    if n_train < max_lookback + 10 {
        eprintln!("ERROR... n_train must be at least 10 greater than max_lookback");
        return;
    }

    // Read market prices
    let prices = match read_market_file(filename) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Cannot open market history file {}: {}", filename, e);
            return;
        }
    };

    println!("Market price history read: {} prices", prices.len());

    if n_train + n_test > prices.len() {
        eprintln!("ERROR... n_train + n_test must not exceed number of prices");
        return;
    }

    let mut returns_open = Vec::new();
    let mut returns_complete = Vec::new();
    let mut returns_grouped = Vec::new();

    let mut train_start = 0;

    // Walkforward analysis
    loop {
        if train_start + n_train > prices.len() {
            break;
        }

        let training_prices = &prices[train_start..train_start + n_train];
        let (crit, lookback, thresh, last_pos) = opt_params(training_prices, max_lookback);

        println!(
            "\nIS at {}  Lookback={}  Thresh={:.3}  Crit={:.3}",
            train_start, lookback, thresh, crit
        );

        let test_start = train_start + n_train;
        let mut n = n_test;
        if n > prices.len() - test_start {
            n = prices.len() - test_start;
        }

        let ret_open = comp_return(1, &prices, test_start, n, lookback, thresh, last_pos);
        let ret_complete = comp_return(2, &prices, test_start, n, lookback, thresh, last_pos);
        let ret_grouped = comp_return(0, &prices, test_start, n, lookback, thresh, last_pos);

        returns_open.extend(&ret_open);
        returns_complete.extend(&ret_complete);
        returns_grouped.extend(&ret_grouped);

        println!(
            "OOS 1 testing {} from {} had {} returns, total={}",
            n,
            test_start,
            ret_open.len(),
            returns_open.len()
        );
        println!(
            "OOS 2 testing {} from {} had {} returns, total={}",
            n,
            test_start,
            ret_complete.len(),
            returns_complete.len()
        );

        train_start += n;
        if train_start + n_train >= prices.len() {
            break;
        }
    }

    // Crunch grouped returns
    let crunch = 10;
    let mut crunched = Vec::new();
    let num_groups = (returns_grouped.len() + crunch - 1) / crunch;

    for i in 0..num_groups {
        let start = i * crunch;
        let end = std::cmp::min(start + crunch, returns_grouped.len());
        let group_mean = returns_grouped[start..end].iter().sum::<f64>() / (end - start) as f64;
        crunched.push(group_mean);
    }

    // Compute statistics
    println!("\n\nnprices={}  max_lookback={}  n_train={}  n_test={}", 
             prices.len(), max_lookback, n_train, n_test);

    let (mean_open, stddev_open, t_open, p_open, t_lower_open) = compute_stats(&returns_open);
    println!(
        "\nOOS mean return per open-trade bar (times 25200) = {:.5}\n  StdDev = {:.5}  t = {:.2}  p = {:.4}  lower = {:.5}  nret={}",
        25200.0 * mean_open,
        25200.0 * stddev_open,
        t_open,
        p_open,
        25200.0 * t_lower_open,
        returns_open.len()
    );

    let (mean_complete, stddev_complete, t_complete, p_complete, t_lower_complete) =
        compute_stats(&returns_complete);
    println!(
        "\nOOS mean return per complete trade (times 1000) = {:.5}\n  StdDev = {:.5}  t = {:.2}  p = {:.4}  lower = {:.5}  nret={}",
        1000.0 * mean_complete,
        1000.0 * stddev_complete,
        t_complete,
        p_complete,
        1000.0 * t_lower_complete,
        returns_complete.len()
    );

    let (mean_grouped, stddev_grouped, t_grouped, p_grouped, t_lower_grouped) =
        compute_stats(&crunched);
    println!(
        "\nOOS mean return per {}-bar group (times 25200) = {:.5}\n  StdDev = {:.5}  t = {:.2}  p = {:.4}  lower = {:.5}  nret={}",
        crunch,
        25200.0 * mean_grouped,
        25200.0 * stddev_grouped,
        t_grouped,
        p_grouped,
        25200.0 * t_lower_grouped,
        crunched.len()
    );

    if returns_open.len() < 2 || returns_complete.len() < 2 || crunched.len() < 2 {
        println!("\nBootstraps skipped due to too few returns");
        println!("Press any key to exit...");
        let _ = io::stdin().read(&mut [0; 1]);
        return;
    }

    // Bootstrap confidence intervals
    println!("\n\nDoing bootstrap 1 of 6...");
    let (_, _, _, _, b1_lower_open) = boot_conf_pctile(&returns_open, find_mean, n_boot);
    let b2_lower_open = 2.0 * mean_open - b1_lower_open;

    println!("Doing bootstrap 2 of 6...");
    let (_, _, _, _, b3_lower_open) = boot_conf_bca(&returns_open, find_mean, n_boot);

    println!("Doing bootstrap 3 of 6...");
    let (_, _, _, _, b1_lower_complete) = boot_conf_pctile(&returns_complete, find_mean, n_boot);
    let b2_lower_complete = 2.0 * mean_complete - b1_lower_complete;

    println!("Doing bootstrap 4 of 6...");
    let (_, _, _, _, b3_lower_complete) = boot_conf_bca(&returns_complete, find_mean, n_boot);

    println!("Doing bootstrap 5 of 6...");
    let (_, _, _, _, b1_lower_grouped) = boot_conf_pctile(&crunched, find_mean, n_boot);
    let b2_lower_grouped = 2.0 * mean_grouped - b1_lower_grouped;

    println!("Doing bootstrap 6 of 6...");
    let (_, _, _, _, b3_lower_grouped) = boot_conf_bca(&crunched, find_mean, n_boot);

    println!("\n\n90 percent lower confidence bounds");
    println!("            Open posn   Complete   Grouped");
    println!(
        "\nStudent's t  {:7.4}    {:7.4}    {:7.4}",
        25200.0 * t_lower_open,
        1000.0 * t_lower_complete,
        25200.0 * t_lower_grouped
    );
    println!(
        "Percentile   {:7.4}    {:7.4}    {:7.4}",
        25200.0 * b1_lower_open,
        1000.0 * b1_lower_complete,
        25200.0 * b1_lower_grouped
    );
    println!(
        "Pivot        {:7.4}    {:7.4}    {:7.4}",
        25200.0 * b2_lower_open,
        1000.0 * b2_lower_complete,
        25200.0 * b2_lower_grouped
    );
    println!(
        "BCa          {:7.4}    {:7.4}    {:7.4}",
        25200.0 * b3_lower_open,
        1000.0 * b3_lower_complete,
        25200.0 * b3_lower_grouped
    );

    println!("\n\nPress any key...");
    let _ = io::stdin().read(&mut [0; 1]);
}