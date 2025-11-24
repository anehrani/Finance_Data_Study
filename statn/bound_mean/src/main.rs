mod boot_conf;
mod qsort;
mod stats;
mod unifrand;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Maximum moving-average lookback
    #[arg(index = 1)]
    max_lookback: usize,

    /// Number of bars in training set
    #[arg(index = 2)]
    n_train: usize,

    /// Number of bars in test set
    #[arg(index = 3)]
    n_test: usize,

    /// Number of bootstrap reps
    #[arg(index = 4)]
    n_boot: usize,

    /// Name of market file (YYYYMMDD Price)
    #[arg(index = 5)]
    filename: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.n_train < args.max_lookback + 10 {
        anyhow::bail!("n_train must be at least 10 greater than max_lookback");
    }

    println!("Reading market file {:?}...", args.filename);
    let prices = read_market_file(&args.filename)?;
    println!("Market price history read. {} records.", prices.len());

    if args.n_train + args.n_test > prices.len() {
        anyhow::bail!("n_train + n_test must not exceed n_prices");
    }

    // Initialize for walkforward
    let mut returns_open = Vec::with_capacity(prices.len());
    let mut returns_complete = Vec::with_capacity(prices.len());
    let mut returns_grouped = Vec::with_capacity(prices.len());

    let mut train_start = 0;
    let mut nret_open = 0;
    let mut nret_complete = 0;
    let mut nret_grouped = 0;

    // Do walkforward
    loop {
        // Train
        let (lookback, thresh, last_pos, crit) = opt_params(
            args.n_train,
            &prices[train_start..],
            args.max_lookback,
        );

        println!(
            " IS at {}  Lookback={}  Thresh={:.3}  Crit={:.3}",
            train_start, lookback, thresh, crit
        );

        let mut n = args.n_test;
        if n > prices.len() - train_start - args.n_train {
            n = prices.len() - train_start - args.n_train;
        }

        // Test with each of the three return types
        let n_returns = comp_return(
            0,
            &prices,
            train_start + args.n_train,
            n,
            lookback,
            thresh,
            last_pos,
            &mut returns_grouped,
        );
        nret_grouped += n_returns;

        println!(
            "OOS 0 testing {} from {} had {} returns, total={}",
            n,
            train_start + args.n_train,
            n_returns,
            nret_grouped
        );

        let n_returns = comp_return(
            1,
            &prices,
            train_start + args.n_train,
            n,
            lookback,
            thresh,
            last_pos,
            &mut returns_open,
        );
        nret_open += n_returns;

        println!(
            "OOS 1 testing {} from {} had {} returns, total={}",
            n,
            train_start + args.n_train,
            n_returns,
            nret_open
        );

        let n_returns = comp_return(
            2,
            &prices,
            train_start + args.n_train,
            n,
            lookback,
            thresh,
            last_pos,
            &mut returns_complete,
        );
        nret_complete += n_returns;

        println!(
            "OOS 2 testing {} from {} had {} returns, total={}",
            n,
            train_start + args.n_train,
            n_returns,
            nret_complete
        );

        // Advance fold window; quit if done
        train_start += n;
        if train_start + args.n_train >= prices.len() {
            break;
        }
    }

    // Crunch the grouped returns
    let crunch = 10;
    let n_returns_crunched = (nret_grouped + crunch - 1) / crunch;
    for i in 0..n_returns_crunched {
        let mut n = crunch;
        if i * crunch + n > nret_grouped {
            n = nret_grouped - i * crunch;
        }
        let mut sum = 0.0;
        for j in i * crunch..i * crunch + n {
            sum += returns_grouped[j];
        }
        returns_grouped[i] = sum / n as f64;
    }
    // Truncate the vector to the new size
    returns_grouped.truncate(n_returns_crunched);
    nret_grouped = n_returns_crunched;

    // Compute and print OOS performance
    println!(
        "\n\nnprices={}  max_lookback={}  n_train={}  n_test={}",
        prices.len(),
        args.max_lookback,
        args.n_train,
        args.n_test
    );

    analyze_returns("Open posn", &returns_open, 25200.0);
    analyze_returns("Complete", &returns_complete, 1000.0);
    analyze_returns("Grouped", &returns_grouped, 25200.0); // Note: C++ uses 25200 for grouped too

    if nret_open < 2 || nret_complete < 2 || nret_grouped < 2 {
        println!("\n\nBootstraps skipped due to too few returns");
        return Ok(());
    }

    // Do bootstraps
    println!("\n\nDoing bootstrap 1 of 6...");
    let (b1_lower_open, _, _, _, _, high_open) = boot_conf::boot_conf_pctile(
        nret_open,
        &returns_open,
        find_mean,
        args.n_boot,
    );
    let mean_open = find_mean(nret_open, &returns_open);
    let b2_lower_open = 2.0 * mean_open - high_open;

    println!("\nDoing bootstrap 2 of 6...");
    let (_, _, _, _, b3_lower_open, _) = boot_conf::boot_conf_bca(
        nret_open,
        &returns_open,
        find_mean,
        args.n_boot,
    );

    println!("\nDoing bootstrap 3 of 6...");
    let (_, _, _, _, b1_lower_complete, high_complete) = boot_conf::boot_conf_pctile(
        nret_complete,
        &returns_complete,
        find_mean,
        args.n_boot,
    );
    let mean_complete = find_mean(nret_complete, &returns_complete);
    let b2_lower_complete = 2.0 * mean_complete - high_complete;

    println!("\nDoing bootstrap 4 of 6...");
    let (_, _, _, _, b3_lower_complete, _) = boot_conf::boot_conf_bca(
        nret_complete,
        &returns_complete,
        find_mean,
        args.n_boot,
    );

    println!("\nDoing bootstrap 5 of 6...");
    let (_, _, _, _, b1_lower_grouped, high_grouped) = boot_conf::boot_conf_pctile(
        nret_grouped,
        &returns_grouped,
        find_mean,
        args.n_boot,
    );
    let mean_grouped = find_mean(nret_grouped, &returns_grouped);
    let b2_lower_grouped = 2.0 * mean_grouped - high_grouped;

    println!("\nDoing bootstrap 6 of 6...");
    let (_, _, _, _, b3_lower_grouped, _) = boot_conf::boot_conf_bca(
        nret_grouped,
        &returns_grouped,
        find_mean,
        args.n_boot,
    );

    // We need t_lower values too, which were computed in analyze_returns but not returned.
    // I should probably refactor analyze_returns to return t_lower.
    
    let t_lower_open = calc_t_lower(&returns_open);
    let t_lower_complete = calc_t_lower(&returns_complete);
    let t_lower_grouped = calc_t_lower(&returns_grouped);

    println!("\n\n90 percent lower confidence bounds");
    println!("            Open posn   Complete   Grouped");
    println!(
        "Student's t  {:7.4}    {:7.4}    {:7.4}",
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

    Ok(())
}

fn opt_params(
    nprices: usize,
    prices: &[f64],
    max_lookback: usize,
) -> (usize, f64, i32, f64) {
    let mut best_perf = -1.0e60;
    let mut ibestlook = 0;
    let mut ibestthresh = 0;
    let mut last_position_of_best = 0;

    for ilook in 2..=max_lookback {
        for ithresh in 1..=10 {
            let mut total_return = 0.0;
            let mut n_trades = 0;
            let mut position = 0;
            let mut ma_sum = 0.0;

            // Initialize MA sum
            // The C++ loop: for (i=max_lookback-1 ; i<nprices-1 ; i++)
            // i is the decision bar index.
            
            // We need to be careful with indices.
            // prices slice starts at train_start.
            // nprices is n_train.
            
            // First valid decision point is at max_lookback - 1.
            // We need history from i-ilook to i.
            
            for i in max_lookback - 1..nprices - 1 {
                if i == max_lookback - 1 {
                    ma_sum = 0.0;
                    for j in (i + 1 - ilook)..=i { // j from i-ilook+1 to i (inclusive) has length ilook
                         // C++: for (j=i ; j>i-ilook ; j--) -> j goes i, i-1, ..., i-ilook+1. Correct.
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

            total_return /= n_trades as f64 + 1.0e-30;
            if total_return > best_perf {
                best_perf = total_return;
                ibestlook = ilook;
                ibestthresh = ithresh;
                last_position_of_best = position;
            }
        }
    }

    (
        ibestlook,
        0.01 * ibestthresh as f64,
        last_position_of_best,
        best_perf,
    )
}

fn comp_return(
    ret_type: i32,
    prices: &[f64],
    istart: usize,
    ntest: usize,
    lookback: usize,
    thresh: f64,
    last_pos: i32,
    returns: &mut Vec<f64>,
) -> usize {
    let mut nret = 0;
    let mut position = last_pos;
    let mut prior_position = 0;
    let trial_thresh = 1.0 + thresh;
    let mut open_price = 0.0;
    let mut ma_sum = 0.0;

    // The loop in C++: for (i=istart-1 ; i<istart-1+ntest ; i++)
    // i is the decision bar.
    
    for i in istart - 1..istart - 1 + ntest {
        if i == istart - 1 {
            ma_sum = 0.0;
            for j in (i - lookback + 1)..=i {
                ma_sum += prices[j];
            }
        } else {
            ma_sum += prices[i] - prices[i - lookback];
        }

        let ma_mean = ma_sum / lookback as f64;

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

        if ret_type == 0 {
            returns.push(ret);
            nret += 1;
        } else if ret_type == 1 {
            if position != 0 {
                returns.push(ret);
                nret += 1;
            }
        } else if ret_type == 2 {
            if position != 0 && prior_position == 0 {
                open_price = prices[i];
            } else if prior_position != 0 && position == 0 {
                returns.push(prices[i] - open_price);
                nret += 1;
            } else if position != 0 && i == istart - 2 + ntest {
                // Force close at end of data
                // C++: i == istart-2+ntest.
                // Loop goes up to < istart-1+ntest, so max i is istart-1+ntest-1 = istart+ntest-2.
                // So this is the last iteration.
                returns.push(prices[i + 1] - open_price);
                nret += 1;
            }
        }

        prior_position = position;
    }

    nret
}

fn find_mean(n: usize, x: &[f64]) -> f64 {
    let mut sum = 0.0;
    for i in 0..n {
        sum += x[i];
    }
    sum / n as f64
}

fn analyze_returns(label: &str, returns: &[f64], scale: f64) {
    let n = returns.len();
    let mean = find_mean(n, returns);
    let mut stddev = 0.0;
    for x in returns {
        let diff = x - mean;
        stddev += diff * diff;
    }

    let (t_val, p_val, t_lower) = if n > 1 {
        let stddev_val = (stddev / (n - 1) as f64).sqrt();
        let t = (n as f64).sqrt() * mean / (stddev_val + 1.0e-20);
        let p = 1.0 - stats::t_cdf((n - 1) as i32, t);
        let t_low = mean - stddev_val / (n as f64).sqrt() * stats::inverse_t_cdf((n - 1) as i32, 0.9);
        (t, p, t_low)
    } else {
        (0.0, 1.0, 0.0)
    };
    
    let stddev_disp = if n > 1 { (stddev / (n - 1) as f64).sqrt() } else { 0.0 };

    println!(
        "OOS mean return per {} (times {}) = {:.5}\n  StdDev = {:.5}  t = {:.2}  p = {:.4}  lower = {:.5}  nret={}",
        label, scale, scale * mean, scale * stddev_disp, t_val, p_val, scale * t_lower, n
    );
}

fn calc_t_lower(returns: &[f64]) -> f64 {
    let n = returns.len();
    if n <= 1 { return 0.0; }
    
    let mean = find_mean(n, returns);
    let mut stddev = 0.0;
    for x in returns {
        let diff = x - mean;
        stddev += diff * diff;
    }
    let stddev_val = (stddev / (n - 1) as f64).sqrt();
    mean - stddev_val / (n as f64).sqrt() * stats::inverse_t_cdf((n - 1) as i32, 0.9)
}

fn read_market_file(filename: &PathBuf) -> Result<Vec<f64>> {
    let file = File::open(filename).context("Cannot open market history file")?;
    let reader = BufReader::new(file);
    let mut prices = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().len() < 2 {
            continue;
        }

        if line.len() < 10 {
             continue;
        }

        let price_str = &line[9..];
        let price_part = price_str.split_whitespace().next().context(format!("Invalid price format at line {}", line_num + 1))?;
        
        let price: f64 = price_part.parse().context(format!("Invalid price value at line {}", line_num + 1))?;
        
        if price > 0.0 {
            prices.push(price.ln());
        } else {
             prices.push(price);
        }
    }

    Ok(prices)
}

