use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use statn::models::cd_ma::{CoordinateDescent, cv_train};

/// Compute moving average crossover indicators
fn compute_indicators(
    nind: usize,
    prices: &[f64],
    start_idx: usize,
    short_term: usize,
    long_term: usize,
) -> Vec<f64> {
    let mut inds = vec![0.0; nind];
    
    for i in 0..nind {
        let k = start_idx + i;
        
        // Compute short-term mean
        let mut short_mean = 0.0;
        for j in 0..short_term {
            short_mean += prices[k - j];
        }
        short_mean /= short_term as f64;
        
        // Compute long-term mean
        let mut long_mean = 0.0;
        for j in 0..long_term {
            long_mean += prices[k - j];
        }
        long_mean /= long_term as f64;
        
        inds[i] = short_mean - long_mean;
    }
    
    inds
}

fn main() {
    println!("CD_MA - Moving Average Crossover Indicator Selection\n");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 6 {
        eprintln!("Usage: try_cd_ma lookback_inc n_long n_short alpha filename");
        eprintln!("  lookback_inc - increment to long-term lookback");
        eprintln!("  n_long - Number of long-term lookbacks");
        eprintln!("  n_short - Number of short-term lookbacks");
        eprintln!("  alpha - Alpha, (0-1]");
        eprintln!("  filename - name of market file (YYYYMMDD Price)");
        std::process::exit(1);
    }
    
    let lookback_inc: usize = args[1].parse().expect("Invalid lookback_inc");
    let n_long: usize = args[2].parse().expect("Invalid n_long");
    let n_short: usize = args[3].parse().expect("Invalid n_short");
    let alpha: f64 = args[4].parse().expect("Invalid alpha");
    let filename = &args[5];
    
    if alpha >= 1.0 {
        eprintln!("Alpha must be less than 1");
        std::process::exit(1);
    }
    
    // Open results file
    let mut fp_results = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("CD_MA.LOG")
        .expect("Cannot create CD_MA.LOG");
    
    writeln!(fp_results, "Starting CD_MA with alpha = {:.4}", alpha).unwrap();
    
    // Read market prices
    println!("Reading market file...");
    
    let file = File::open(filename).expect("Cannot open market history file");
    let reader = BufReader::new(file);
    
    let mut prices = Vec::new();
    
    for line in reader.lines() {
        let line = line.expect("Error reading line");
        if line.len() < 2 {
            continue;
        }
        
        // Parse date (first 8 characters should be digits)
        if line.len() < 9 {
            continue;
        }
        
        // Parse price (after date)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        
        if let Ok(price) = parts[1].parse::<f64>() {
            if price > 0.0 {
                prices.push(price.ln());
            }
        }
    }
    
    let nprices = prices.len();
    println!("Market price history read: {} prices", nprices);
    writeln!(fp_results, "Read {} prices from {}", nprices, filename).unwrap();
    
    // Initialize computation parameters
    let n_lambdas = 50;
    let nvars = n_long * n_short;
    let n_test = 252; // One year
    
    let max_lookback = n_long * lookback_inc;
    let n_train = nprices - n_test - max_lookback;
    
    if n_train < nvars + 10 {
        eprintln!("ERROR: Too little training data for parameters");
        std::process::exit(1);
    }
    
    println!("Training cases: {}", n_train);
    println!("Test cases: {}", n_test);
    println!("Number of indicators: {}", nvars);
    
    // Allocate arrays
    let max_cases = n_train.max(n_test);
    let mut data = vec![0.0; max_cases * nvars];
    let mut targets = vec![0.0; max_cases];
    let mut lambdas = vec![0.0; n_lambdas];
    let mut lambda_oos = vec![0.0; n_lambdas];
    
    // Compute and save indicators for training set
    println!("Computing training indicators...");
    
    let mut k = 0;
    for ilong in 0..n_long {
        let long_lookback = (ilong + 1) * lookback_inc;
        for ishort in 0..n_short {
            let short_lookback = long_lookback * (ishort + 1) / (n_short + 1);
            let short_lookback = short_lookback.max(1);
            
            let inds = compute_indicators(
                n_train,
                &prices,
                max_lookback - 1,
                short_lookback,
                long_lookback,
            );
            
            for i in 0..n_train {
                data[i * nvars + k] = inds[i];
            }
            k += 1;
        }
    }
    
    // Compute targets for training set
    for i in 0..n_train {
        let idx = max_lookback - 1 + i;
        targets[i] = prices[idx + 1] - prices[idx];
    }
    
    // Run cross-validation to find optimal lambda
    println!("Running cross-validation...");
    
    let lambda = if alpha <= 0.0 {
        writeln!(fp_results, "\nUser specified negative alpha, so lambda = 0").unwrap();
        0.0
    } else {
        let best_lambda = cv_train(
            nvars,
            10, // 10-fold CV
            &data[..n_train * nvars],
            &targets[..n_train],
            None,
            &mut lambdas,
            &mut lambda_oos,
            true, // covar_updates
            n_lambdas,
            alpha,
            1000, // maxits
            1.0e-9,
            true, // fast_test
        );
        
        writeln!(fp_results, "\nCross validation gave optimal lambda = {:.4}", best_lambda).unwrap();
        writeln!(fp_results, "  Lambda   OOS explained").unwrap();
        for i in 0..n_lambdas {
            writeln!(fp_results, "{:8.4} {:12.4}", lambdas[i], lambda_oos[i]).unwrap();
        }
        
        best_lambda
    };
    
    // Train the final model
    println!("Training final model with lambda = {:.6}...", lambda);
    
    let mut cd = CoordinateDescent::new(nvars, n_train, false, true, 0);
    cd.get_data(0, n_train, &data[..n_train * nvars], &targets[..n_train], None);
    cd.core_train(alpha, lambda, 1000, 1.0e-7, true, false);
    
    writeln!(
        fp_results,
        "\nBetas, with in-sample explained variance = {:.5} percent",
        100.0 * cd.explained
    )
    .unwrap();
    writeln!(
        fp_results,
        "Row label is long-term lookback; Columns run from smallest to largest short-term lookback"
    )
    .unwrap();
    
    // Print beta coefficients
    k = 0;
    for ilong in 0..n_long {
        let long_lookback = (ilong + 1) * lookback_inc;
        write!(fp_results, "\n{:5} ", long_lookback).unwrap();
        for _ishort in 0..n_short {
            if cd.beta[k] != 0.0 {
                write!(fp_results, "{:9.4}", cd.beta[k]).unwrap();
            } else {
                write!(fp_results, "    ---- ").unwrap();
            }
            k += 1;
        }
    }
    writeln!(fp_results).unwrap();
    
    // Compute indicators for test set
    println!("Computing test indicators...");
    
    k = 0;
    for ilong in 0..n_long {
        let long_lookback = (ilong + 1) * lookback_inc;
        for ishort in 0..n_short {
            let short_lookback = long_lookback * (ishort + 1) / (n_short + 1);
            let short_lookback = short_lookback.max(1);
            
            let inds = compute_indicators(
                n_test,
                &prices,
                n_train + max_lookback - 1,
                short_lookback,
                long_lookback,
            );
            
            for i in 0..n_test {
                data[i * nvars + k] = inds[i];
            }
            k += 1;
        }
    }
    
    // Compute targets for test set
    for i in 0..n_test {
        let idx = n_train + max_lookback - 1 + i;
        targets[i] = prices[idx + 1] - prices[idx];
    }
    
    // Evaluate on test set
    println!("Evaluating on test set...");
    
    let mut sum = 0.0;
    for i in 0..n_test {
        let xptr = &data[i * nvars..(i + 1) * nvars];
        let mut pred = 0.0;
        for ivar in 0..nvars {
            pred += cd.beta[ivar] * (xptr[ivar] - cd.xmeans[ivar]) / cd.xscales[ivar];
        }
        pred = pred * cd.yscale + cd.ymean;
        
        if pred > 0.0 {
            sum += targets[i];
        } else if pred < 0.0 {
            sum -= targets[i];
        }
    }
    
    writeln!(
        fp_results,
        "\nOOS total return = {:.5} ({:.3} percent)",
        sum,
        100.0 * (sum.exp() - 1.0)
    )
    .unwrap();
    
    println!("\nResults:");
    println!("  In-sample explained variance: {:.3}%", 100.0 * cd.explained);
    println!("  OOS total return: {:.5} ({:.3}%)", sum, 100.0 * (sum.exp() - 1.0));
    println!("\nResults written to CD_MA.LOG");
}
