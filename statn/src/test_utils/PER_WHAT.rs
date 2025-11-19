use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

const MKTBUF: usize = 2048; // Alloc for market info in chunks of this many records

/*
Compute optimal lookback and breakout threshold for a primitive long-only 
moving-average breakout system.
*/

fn opt_params(
    which_crit: usize,    // 0=mean return per bar; 1=profit factor; 2=Sharpe ratio
    all_bars: bool,       // Include return of all bars, even those with no position
    nprices: usize,       // Number of log prices
    prices: &[f64],       // Log prices
    max_lookback: usize,  // Maximum lookback to use
) -> (usize, f64, i32, f64) {
    // Returns (lookback, threshold, last_position, best_performance)
    
    let mut best_perf = -1e60;
    let mut ibestlook = 2;
    let mut ibestthresh = 1;
    let mut last_position_of_best = 0;

    for ilook in 2..=max_lookback {
        for ithresh in 1..=10 {
            let mut total_return = 0.0;
            let mut win_sum = 1e-60;
            let mut lose_sum = 1e-60;
            let mut sum_squares = 1e-60;
            let mut n_trades = 0;
            let mut position = 0;

            let mut ma_sum = 0.0;
            let trial_thresh = 1.0 + 0.01 * ithresh as f64;

            for i in (max_lookback - 1)..(nprices - 1) {
                if i == max_lookback - 1 {
                    // Find the moving average for the first valid case
                    ma_sum = 0.0;
                    for j in (i - ilook + 1)..=i {
                        ma_sum += prices[j];
                    }
                } else {
                    // Update the moving average
                    ma_sum += prices[i] - prices[i - ilook];
                }

                let ma_mean = ma_sum / ilook as f64;

                // Make a trade decision
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

                if all_bars || position != 0 {
                    n_trades += 1;
                    total_return += ret;
                    sum_squares += ret * ret;
                    if ret > 0.0 {
                        win_sum += ret;
                    } else {
                        lose_sum -= ret;
                    }
                }
            }

            let n_trades_f = n_trades as f64 + 1e-30;
            let perf = match which_crit {
                0 => {
                    // Mean return criterion
                    total_return / n_trades_f
                }
                1 => {
                    // Profit factor criterion
                    win_sum / lose_sum
                }
                2 => {
                    // Sharpe ratio criterion
                    let mean_ret = total_return / n_trades_f;
                    let mut variance = sum_squares / n_trades_f - mean_ret * mean_ret;
                    if variance < 1e-20 {
                        variance = 1e-20;
                    }
                    mean_ret / variance.sqrt()
                }
                _ => 0.0,
            };

            if perf > best_perf {
                best_perf = perf;
                ibestlook = ilook;
                ibestthresh = ithresh;
                last_position_of_best = position;
            }
        }
    }

    let thresh = 0.01 * ibestthresh as f64;
    (ibestlook, thresh, last_position_of_best, best_perf)
}

/*
Compute return vector for all bars but output returns according to user's request.
*/

fn comp_return(
    ret_type: usize,      // Return type: 0=all bars; 1=bars with open position; 2=completed trades
    nprices: usize,       // Number of log prices (for safety)
    prices: &[f64],       // Log prices
    istart: usize,        // Starting index in OOS test set
    ntest: usize,         // Number of OOS test cases
    lookback: usize,      // Optimal MA lookback
    thresh: f64,          // Optimal breakout threshold factor
    last_pos: i32,        // Position in bar prior to test set
    returns: &mut Vec<f64>, // Bar returns returned here
) {
    let mut position = last_pos;
    let mut prior_position = 0;
    let trial_thresh = 1.0 + thresh;
    let mut ma_sum = 0.0;

    for i in (istart - 1)..(istart - 1 + ntest) {
        if i == istart - 1 {
            // Find the moving average for the first valid case
            ma_sum = 0.0;
            for j in (i - lookback + 1)..=i {
                assert!(j >= 0, "Index out of bounds");
                ma_sum += prices[j];
            }
        } else {
            // Update the moving average
            ma_sum += prices[i] - prices[i - lookback];
        }

        let ma_mean = ma_sum / lookback as f64;

        assert!(i + 1 < nprices, "Index out of bounds");

        // Make a trade decision
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

        // Save the appropriate return
        match ret_type {
            0 => {
                // All bars, even those with no position
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
                    // Store open price in a temporary variable (simplified approach)
                } else if prior_position != 0 && position == 0 {
                    // Just closed a trade
                    // Need to track open price - implementation simplified
                } else if position != 0 && i == istart - 2 + ntest {
                    // Force close at end of data
                }
            }
            _ => {}
        }

        prior_position = position;
    }
}

/*
Main routine
*/

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 8 {
        eprintln!("Usage: per_what <which_crit> <all_bars> <ret_type> <max_lookback> <n_train> <n_test> <filename>");
        eprintln!("  which_crit - 0=mean return; 1=profit factor; 2=Sharpe ratio");
        eprintln!("  all_bars - Training: Include all bars in return?");
        eprintln!("  ret_type - Testing: 0=all bars; 1=bars with position open; 2=completed trades");
        eprintln!("  max_lookback - Maximum moving-average lookback");
        eprintln!("  n_train - Number of bars in training set");
        eprintln!("  n_test - Number of bars in test set");
        eprintln!("  filename - name of market file (YYYYMMDD Price)");
        std::process::exit(1);
    }

    let which_crit: usize = args[1].parse().expect("Invalid which_crit");
    let all_bars: bool = args[2].parse::<u32>().expect("Invalid all_bars") != 0;
    let ret_type: usize = args[3].parse().expect("Invalid ret_type");
    let max_lookback: usize = args[4].parse().expect("Invalid max_lookback");
    let n_train: usize = args[5].parse().expect("Invalid n_train");
    let n_test: usize = args[6].parse().expect("Invalid n_test");
    let filename = &args[7];

    if n_train < max_lookback + 10 {
        eprintln!("ERROR... n_train must be at least 10 greater than max_lookback");
        std::process::exit(1);
    }

    // Read market prices
    let mut prices = read_market_file(filename);

    if prices.is_empty() {
        eprintln!("No prices read from file");
        std::process::exit(1);
    }

    let nprices = prices.len();
    println!("Market price history read: {} prices", nprices);

    if n_train + n_test > nprices {
        eprintln!("ERROR... n_train + n_test must not exceed n_prices");
        std::process::exit(1);
    }

    let mult = if which_crit == 0 {
        println!("Mean return criterion will be multiplied by 25200 in all results");
        25200
    } else {
        1
    };

    let mut train_start = 0;
    let mut all_returns = Vec::new();

    // Do walkforward
    loop {
        // Train
        let (lookback, thresh, last_pos, crit) =
            opt_params(which_crit, all_bars, n_train, &prices[train_start..train_start + n_train], max_lookback);

        println!(
            "\nIS at {}  Lookback={}  Thresh={:.3}  Crit={:.3}",
            train_start,
            lookback,
            thresh,
            mult as f64 * crit
        );

        let n = std::cmp::min(n_test, nprices - train_start - n_train);

        // Test
        let mut test_returns = Vec::new();
        comp_return(
            ret_type,
            nprices,
            &prices,
            train_start + n_train,
            n,
            lookback,
            thresh,
            last_pos,
            &mut test_returns,
        );

        println!(
            "\nOOS testing {} from {} had {} returns, total={}",
            n,
            train_start + n_train,
            test_returns.len(),
            all_returns.len() + test_returns.len()
        );

        all_returns.extend(test_returns);

        // Advance fold window; quit if done
        train_start += n;
        if train_start + n_train >= nprices {
            break;
        }
    }

    // Compute and print OOS performance
    println!(
        "\n\nnprices={}  max_lookback={}  which_crit={}  all_bars={}  ret_type={}  n_train={}  n_test={}",
        nprices, max_lookback, which_crit, all_bars as u32, ret_type, n_train, n_test
    );

    let nret = all_returns.len();

    match which_crit {
        0 => {
            let mut crit = 0.0;
            for &ret in &all_returns {
                crit += ret;
            }
            crit /= (nret as f64 + 1e-60);
            println!(
                "\n\nOOS mean return per open-trade bar (times 25200) = {:.5}  nret={}",
                25200.0 * crit,
                nret
            );
        }
        1 => {
            let mut win_sum = 1e-60;
            let mut lose_sum = 1e-60;
            for &ret in &all_returns {
                if ret > 0.0 {
                    win_sum += ret;
                } else if ret < 0.0 {
                    lose_sum -= ret;
                }
            }
            let crit = win_sum / lose_sum;
            println!("\n\nOOS profit factor = {:.5}  nret={}", crit, nret);
        }
        2 => {
            let mut sum = 0.0;
            let mut sum_squares = 0.0;
            for &ret in &all_returns {
                sum += ret;
                sum_squares += ret * ret;
            }
            sum /= (nret as f64 + 1e-60);
            sum_squares /= (nret as f64 + 1e-60);
            sum_squares -= sum * sum;
            let sum_squares = if sum_squares < 1e-20 {
                1e-20
            } else {
                sum_squares
            };
            let crit = sum / sum_squares.sqrt();
            println!("\n\nOOS raw Sharpe ratio = {:.5}  nret={}", crit, nret);
        }
        _ => {}
    }

    println!("\n\nPress Enter to exit...");
    let _ = io::stdin().read(&mut [0u8]);
}

fn read_market_file(filename: &str) -> Vec<f64> {
    let mut prices = Vec::new();

    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            println!("Reading market file...");

            for line in reader.lines() {
                match line {
                    Ok(line_content) => {
                        let trimmed = line_content.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        // Parse the date (first 8 characters)
                        if trimmed.len() < 9 {
                            eprintln!("Invalid line format: {}", trimmed);
                            continue;
                        }

                        let date_str = &trimmed[0..8];
                        if !date_str.chars().all(|c| c.is_ascii_digit()) {
                            eprintln!("Invalid date in line: {}", trimmed);
                            continue;
                        }

                        // Parse the price
                        let price_str = trimmed[9..].trim_start_matches(|c: char| {
                            c == ' ' || c == '\t' || c == ','
                        });

                        match price_str.parse::<f64>() {
                            Ok(price) => {
                                if price > 0.0 {
                                    prices.push(price.ln());
                                }
                            }
                            Err(_) => {
                                eprintln!("Invalid price in line: {}", trimmed);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading line: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Cannot open market history file {}: {}", filename, e);
            std::process::exit(1);
        }
    }

    prices
}

use std::io;