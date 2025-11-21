use std::env;
use statn::io::read_market_file;
use statn::models::CDmodel::cv_train;

fn main() {
    println!("CD Model Cross-Validation Demo\n");
    
    // Get command line arguments or use defaults
    let args: Vec<String> = env::args().collect();
    let data_file = if args.len() > 1 {
        &args[1]
    } else {
        "../data/XAGUSD.txt"
    };
    
    // Read market data
    println!("Reading market data from: {}", data_file);
    let bars = match read_market_file(data_file) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };
    
    let n = bars.len();
    println!("Loaded {} bars\n", n);
    
    // Prepare data for CDmodel
    // We'll use close prices as features and predict next-day returns
    let lookback = 50; // Number of lagged features
    let nvars = lookback;
    let ncases = n - lookback - 1;
    
    if ncases < 20 {
        eprintln!("Not enough data points. Need at least {} bars.", lookback + 21);
        std::process::exit(1);
    }
    
    // Create feature matrix (X) and target vector (y)
    let mut xx = vec![0.0; ncases * nvars];
    let mut yy = vec![0.0; ncases];
    
    for i in 0..ncases {
        // Features: lagged returns
        for j in 0..lookback {
            let idx = i + j;
            let ret = if bars.close[idx + 1] > 0.0 {
                (bars.close[idx + 1] - bars.close[idx]) / bars.close[idx]
            } else {
                0.0
            };
            xx[i * nvars + j] = ret;
        }
        
        // Target: next-day return
        let target_idx = i + lookback;
        yy[i] = if bars.close[target_idx + 1] > 0.0 {
            (bars.close[target_idx + 1] - bars.close[target_idx]) / bars.close[target_idx]
        } else {
            0.0
        };
    }
    
    // Cross-validation parameters
    let nfolds = 5;
    let n_lambda = 20;
    let alpha = 0.1; // Elastic net mixing parameter (0=ridge, 1=lasso)
    let maxits = 2000;
    let eps = 1.0e-6;
    let fast_test = false;
    let covar_updates = true;
    
    let mut lambdas = vec![0.0; n_lambda];
    let mut lambda_oos = vec![0.0; n_lambda];
    
    println!("Running {}-fold cross-validation...", nfolds);
    println!("Number of features: {}", nvars);
    println!("Number of cases: {}", ncases);
    println!("Alpha (elastic net): {}", alpha);
    println!("Number of lambda values: {}\n", n_lambda);
    
    // Run cross-validation
    let best_lambda = cv_train(
        nvars,
        nfolds,
        &xx,
        &yy,
        None, // No weights
        &mut lambdas,
        &mut lambda_oos,
        covar_updates,
        n_lambda,
        alpha,
        maxits,
        eps,
        fast_test,
    );
    
    println!("\nCross-validation complete!");
    println!("Best lambda: {:.6}", best_lambda);
    
    // Display results for all lambda values
    println!("\nLambda values and OOS R²:");
    println!("{:>12} {:>12}", "Lambda", "OOS R²");
    println!("{}", "-".repeat(26));
    for i in 0..n_lambda {
        println!("{:12.6} {:12.6}", lambdas[i], lambda_oos[i]);
    }

    // println!("Coefficients at best λ:");
    // for (i, &b) in best_beta.iter().enumerate() {
    //     println!("  Feature {}: {:.6}", i, b);
    // }
    
    // Find best performing lambda
    let mut best_r2 = lambda_oos[0];
    let mut best_idx = 0;
    for i in 1..n_lambda {
        if lambda_oos[i] > best_r2 {
            best_r2 = lambda_oos[i];
            best_idx = i;
        }
    }
    
    println!("\nBest performance:");
    println!("  Lambda: {:.6}", lambdas[best_idx]);
    println!("  OOS R²: {:.6}", best_r2);
}