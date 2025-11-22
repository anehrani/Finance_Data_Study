use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;

use statn::estimators::sensitivity::sensitivity;
use statn::estimators::StocBias;
use statn::models::differential_evolution::diff_ev;

// Global/Shared data for the criterion function
struct MarketData {
    prices: Vec<f64>,
    max_lookback: usize,
}

/// Evaluate a thresholded moving-average crossover system
fn test_system(
    prices: &[f64],
    max_lookback: usize,
    long_term: usize,
    short_pct: f64,
    short_thresh: f64,
    long_thresh: f64,
    returns: Option<&mut [f64]>,
) -> (f64, i32) {
    let ncases = prices.len();
    let short_term = (0.01 * short_pct * long_term as f64) as usize;
    let short_term = short_term.max(1).min(long_term - 1);
    
    let short_thresh = short_thresh / 10000.0;
    let long_thresh = long_thresh / 10000.0;

    let mut sum = 0.0;
    let mut ntrades = 0;
    
    match returns {
        Some(ret_slice) => {
            let mut ret_idx = 0;
            for i in (max_lookback - 1)..(ncases - 1) {
                let mut short_mean = 0.0;
                for j in (i + 1 - short_term)..=i {
                    short_mean += prices[j];
                }
                
                let mut long_mean = short_mean;
                for j in (i + 1 - long_term)..(i + 1 - short_term) {
                     long_mean += prices[j];
                }
                
                short_mean /= short_term as f64;
                long_mean /= long_term as f64;
                
                let change = short_mean / long_mean - 1.0;
                
                let ret = if change > long_thresh {
                    ntrades += 1;
                    prices[i+1] - prices[i]
                } else if change < -short_thresh {
                    ntrades += 1;
                    prices[i] - prices[i+1]
                } else {
                    0.0
                };
                
                sum += ret;
                if ret_idx < ret_slice.len() {
                    ret_slice[ret_idx] = ret;
                    ret_idx += 1;
                }
            }
        }
        None => {
             for i in (max_lookback - 1)..(ncases - 1) {
                let mut short_mean = 0.0;
                for j in (i + 1 - short_term)..=i {
                    short_mean += prices[j];
                }
                
                let mut long_mean = short_mean;
                for j in (i + 1 - long_term)..(i + 1 - short_term) {
                     long_mean += prices[j];
                }
                
                short_mean /= short_term as f64;
                long_mean /= long_term as f64;
                
                let change = short_mean / long_mean - 1.0;
                
                let ret = if change > long_thresh {
                    ntrades += 1;
                    prices[i+1] - prices[i]
                } else if change < -short_thresh {
                    ntrades += 1;
                    prices[i] - prices[i+1]
                } else {
                    0.0
                };
                
                sum += ret;
            }
        }
    }

    (sum, ntrades)
}

/// Criterion function
fn criter(
    params: &[f64],
    mintrades: i32,
    data: &MarketData,
    stoc_bias: &mut Option<&mut StocBias>,
) -> f64 {
    let long_term = (params[0] + 1.0e-10) as usize;
    let short_pct = params[1];
    let short_thresh = params[2];
    let long_thresh = params[3];

    let (ret_val, ntrades) = if let Some(sb) = stoc_bias {
        let returns = sb.returns_mut();
        test_system(
            &data.prices,
            data.max_lookback,
            long_term,
            short_pct,
            short_thresh,
            long_thresh,
            Some(returns),
        )
    } else {
        test_system(
            &data.prices,
            data.max_lookback,
            long_term,
            short_pct,
            short_thresh,
            long_thresh,
            None,
        )
    };

    if let Some(sb) = stoc_bias {
        if ret_val > 0.0 {
            sb.process();
        }
    }

    if ntrades >= mintrades {
        ret_val
    } else {
        -1.0e20
    }
}


fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let (max_lookback, max_thresh, filename, verbose) = if args.len() >= 4 {
        let max_lookback = args[1].parse::<usize>().unwrap_or(100);
        let max_thresh = args[2].parse::<f64>().unwrap_or(100.0);
        let filename = args[3].clone();
        // Optional 4th argument for verbose mode (default: false)
        let verbose = if args.len() >= 5 {
            args[4].parse::<bool>().unwrap_or(false) || args[4].eq_ignore_ascii_case("true") || args[4] == "1"
        } else {
            false
        };
        (max_lookback, max_thresh, filename, verbose)
    } else {
        println!("\nUsage: try_diff_ev  max_lookback  max_thresh  filename  [verbose]");
        println!("  max_lookback - Maximum moving-average lookback");
        println!("  max_thresh - Maximum fraction threshold times 10000");
        println!("  filename - name of market file (YYYYMMDD Price)");
        println!("  verbose - Optional: true/false to show detailed progress (default: false)");
        // Default values for testing if no args
        (100, 100.0, "test_data.txt".to_string(), false)
    };

    println!("\nConfiguration:");
    println!("  Max lookback: {}", max_lookback);
    println!("  Max threshold: {}", max_thresh);
    println!("  Data file: {}", filename);
    println!("  Verbose mode: {}\n", verbose);


    // Read market prices
    let path = Path::new(&filename);
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            println!("\n\nCannot open market history file {}: {}", filename, e);
            process::exit(1);
        }
    };
    let reader = io::BufReader::new(file);

    let mut prices = Vec::new();
    println!("\nReading market file...");

    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                println!("\nError reading line {} of file {}", line_num + 1, filename);
                process::exit(1);
            }
        };
        
        if line.len() < 2 {
            continue;
        }

        // Parse: YYYYMMDD price1 price2 price3 price4
        // We want the last price (close price)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            // Take the last column as the close price
            if let Ok(price) = parts[parts.len() - 1].parse::<f64>() {
                if price > 0.0 {
                    prices.push(price.ln());
                }
            }
        }
    }

    println!("\nMarket price history read, {} prices", prices.len());
    
    if prices.len() <= max_lookback {
        println!("Not enough prices for max_lookback");
        process::exit(1);
    }

    let market_data = MarketData {
        prices,
        max_lookback,
    };

    let low_bounds = vec![2.0, 0.01, 0.0, 0.0];
    let high_bounds = vec![max_lookback as f64, 99.0, max_thresh, max_thresh];
    let mintrades = 20;

    // Initialize StocBias
    let mut stoc_bias_opt = StocBias::new(market_data.prices.len() - max_lookback);
    if stoc_bias_opt.is_none() {
        println!("Insufficient memory for StocBias");
        process::exit(1);
    }

    // Create a raw pointer to share StocBias between diff_ev and criter_wrapper
    // This is safe because we control the lifetime and ensure no aliasing issues
    let sb_ptr = stoc_bias_opt.as_mut().unwrap() as *mut statn::estimators::StocBias;

    // Create a closure that uses the raw pointer
    let criter_wrapper = |params: &[f64], mintrades: i32| -> f64 {
        unsafe {
            let mut sb_ref = Some(&mut *sb_ptr);
            criter(params, mintrades, &market_data, &mut sb_ref)
        }
    };

    // Run diff_ev - it will set collecting mode automatically
    let result = diff_ev(
        criter_wrapper,
        4,
        1,
        100,
        10000,
        mintrades,
        10000000,
        300,
        0.2,
        0.2,
        0.3,
        &low_bounds,
        &high_bounds,
        verbose,  // Use command-line argument
        &mut stoc_bias_opt,
    );




    match result {
        Ok(params) => {
            println!("\n\nBest performance = {:.4}  Variables follow...", params[4]);
            for i in 0..4 {
                println!("\n  {:.4}", params[i]);
            }

            // Compute and print stochastic bias estimate
            if let Some(ref sb) = stoc_bias_opt {
                let (is_mean, oos_mean, bias) = sb.compute();
                println!("\n\nVery rough estimates from differential evolution initialization...");
                println!("\n  In-sample mean = {:.4}", is_mean);
                println!("\n  Out-of-sample mean = {:.4}", oos_mean);
                println!("\n  Bias = {:.4}", bias);
                println!("\n  Expected = {:.4}", params[4] - bias);
            }

            
            // Sensitivity
            let _ = sensitivity(
                |p, m| criter(p, m, &market_data, &mut None),
                4,
                1,
                30,
                80,
                mintrades,
                &params,
                &low_bounds,
                &high_bounds,
            );
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    println!("\n\n Completed...");
}
