use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process;

use statn::estimators::sensitivity::sensitivity;
use statn::estimators::StocBias;
use statn::models::differential_evolution::diff_ev;

// Import library utilities
use try_diff_ev::config::Config;
use try_diff_ev::{backtest_signals, generate_signals, visualise_signals};

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

/// Load parameters from a file
fn load_parameters(filename: &PathBuf) -> Result<Vec<f64>, String> {
    let file = File::open(filename).map_err(|e| format!("Cannot open file: {}", e))?;
    let reader = io::BufReader::new(file);
    let mut params = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Error reading line: {}", e))?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let val = trimmed.parse::<f64>().map_err(|e| format!("Parse error: {}", e))?;
            params.push(val);
        }
    }
    Ok(params)
}

/// Save parameters to a file
fn save_params_to_file(filename: &PathBuf, params: &[f64]) -> Result<(), String> {
    use std::io::Write;
    let mut file = File::create(filename).map_err(|e| format!("Cannot create file: {}", e))?;
    for param in params {
        writeln!(file, "{}", param).map_err(|e| format!("Write error: {}", e))?;
    }
    Ok(())
}

/// Load market data from file
fn load_market_data(config: &Config) -> Result<MarketData, String> {
    let file = File::open(&config.market.data_file)
        .map_err(|e| format!("Cannot open market file: {}", e))?;
    let reader = io::BufReader::new(file);
    let mut prices = Vec::new();
    
    for line in reader.lines() {
        if let Ok(l) = line {
            if l.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(p) = parts[parts.len() - 1].parse::<f64>() {
                    if p > 0.0 {
                        prices.push(p.ln());
                    }
                }
            }
        }
    }
    
    if prices.len() <= config.market.max_lookback {
        return Err("Not enough data for the requested lookback".to_string());
    }
    
    Ok(MarketData {
        prices,
        max_lookback: config.market.max_lookback,
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: try_diff_ev <config_file>");
        eprintln!("Example: try_diff_ev config.toml");
        process::exit(1);
    }
    
    // Load configuration
    let config = match Config::from_file(&args[1]) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config file: {}", e);
            process::exit(1);
        }
    };
    
    println!("\n=== Configuration ===");
    println!("Mode: {}", config.mode);
    println!("Data file: {}", config.market.data_file.display());
    println!("Max lookback: {}", config.market.max_lookback);
    println!("Output dir: {}\n", config.output.output_dir.display());
    
    // Load market data
    let market_data = match load_market_data(&config) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading market data: {}", e);
            process::exit(1);
        }
    };
    
    println!("Market price history loaded: {} prices\n", market_data.prices.len());
    
    // Create output directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&config.output.output_dir) {
        eprintln!("Error creating output directory: {}", e);
        process::exit(1);
    }
    
    match config.mode.as_str() {
        "optimize" => run_optimization(&config, &market_data),
        "predict" => run_prediction(&config, &market_data),
        _ => {
            eprintln!("Invalid mode: {}. Use 'optimize' or 'predict'.", config.mode);
            process::exit(1);
        }
    }
    
    println!("\n✓ Completed successfully!");
}

fn run_optimization(config: &Config, market_data: &MarketData) {
    let low_bounds = vec![2.0, 0.01, 0.0, 0.0];
    let high_bounds = vec![
        config.market.max_lookback as f64,
        99.0,
        config.market.max_thresh,
        config.market.max_thresh,
    ];
    
    // Initialize StocBias
    let mut stoc_bias_opt = StocBias::new(market_data.prices.len() - market_data.max_lookback);
    if stoc_bias_opt.is_none() {
        eprintln!("Insufficient memory for StocBias");
        process::exit(1);
    }
    
    let sb_ptr = stoc_bias_opt.as_mut().unwrap() as *mut statn::estimators::StocBias;
    let criter_wrapper = |params: &[f64], mintrades: i32| -> f64 {
        unsafe {
            let mut sb_ref = Some(&mut *sb_ptr);
            criter(params, mintrades, market_data, &mut sb_ref)
        }
    };
    
    println!("Running differential evolution optimization...");
    let result = diff_ev(
        criter_wrapper,
        4,
        1,
        100,
        config.optimization.max_gens,
        config.optimization.min_trades,
        10000000,
        config.optimization.popsize,
        0.2,
        0.2,
        0.3,
        &low_bounds,
        &high_bounds,
        config.optimization.verbose,
        &mut stoc_bias_opt,
    );
    
    match result {
        Ok(params) => {
            println!("\n=== OPTIMIZATION RESULTS ===");
            println!("Best performance: {:.4}", params[4]);
            println!("\nOptimal parameters:");
            println!("  Long lookback:  {:.4}", params[0]);
            println!("  Short %:        {:.4}", params[1]);
            println!("  Short thresh:   {:.4}", params[2]);
            println!("  Long thresh:    {:.4}", params[3]);
            
            if let Some(ref sb) = stoc_bias_opt {
                let (is_mean, oos_mean, bias) = sb.compute();
                println!("\nBias estimates:");
                println!("  In-sample mean:     {:.4}", is_mean);
                println!("  Out-of-sample mean: {:.4}", oos_mean);
                println!("  Bias:               {:.4}", bias);
                println!("  Expected return:    {:.4}", params[4] - bias);
            }
            
            // Save parameters if specified
            if let Some(ref params_file) = config.optimization.params_file {
                let output_path = config.output.output_dir.join(params_file);
                if let Err(e) = save_params_to_file(&output_path, &params[0..4]) {
                    eprintln!("Error saving parameters: {}", e);
                } else {
                    println!("\nParameters saved to: {}", output_path.display());
                }
            }
            
            // Run sensitivity analysis
            println!("\nRunning sensitivity analysis...");
            let _ = sensitivity(
                |p, m| criter(p, m, market_data, &mut None),
                4,
                1,
                30,
                80,
                config.optimization.min_trades,
                &params,
                &low_bounds,
                &high_bounds,
            );
            println!("Sensitivity analysis saved to SENS.LOG");
        }
        Err(e) => {
            eprintln!("Optimization error: {}", e);
            process::exit(1);
        }
    }
}

fn run_prediction(config: &Config, market_data: &MarketData) {
    // Load parameters
    let params_path = config.output.output_dir.join(&config.backtest.params_file);
    let params = match load_parameters(&params_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error loading parameters from {}: {}", params_path.display(), e);
            process::exit(1);
        }
    };
    
    if params.len() < 4 {
        eprintln!("Parameters file must contain at least 4 parameters");
        process::exit(1);
    }
    
    println!("=== LOADED PARAMETERS ===");
    println!("  Long lookback:  {:.4}", params[0]);
    println!("  Short %:        {:.4}", params[1]);
    println!("  Short thresh:   {:.4}", params[2]);
    println!("  Long thresh:    {:.4}", params[3]);
    
    // Generate signals
    let result = generate_signals(
        &market_data.prices,
        (params[0] + 1.0e-10) as usize,
        params[1],
        params[2],
        params[3],
    );
    
    // Print last 20 signals
    println!("\nLast 20 signals:");
    let start = result.signals.len().saturating_sub(20);
    for i in start..result.signals.len() {
        let sig = match result.signals[i] {
            1 => "BUY",
            -1 => "SELL",
            _ => "HOLD",
        };
        println!("{:>5}: price={:.4} -> {}", i, result.prices[i], sig);
    }
    
    // Backtest the strategy
    let stats = backtest_signals(
        &result,
        config.backtest.initial_budget,
        config.backtest.transaction_cost_pct,
    );
    
    println!("\n=== BACKTEST RESULTS ===");
    println!("Initial Budget:    ${:.2}", stats.initial_budget);
    println!("Final Budget:      ${:.2}", stats.final_budget);
    println!("Total P&L:         ${:.2}", stats.total_pnl);
    println!("ROI:               {:.2}%", stats.roi_percent);
    println!("\nTrading Statistics:");
    println!("  Total Trades:    {}", stats.num_trades);
    println!("  Winning Trades:  {}", stats.num_wins);
    println!("  Losing Trades:   {}", stats.num_losses);
    println!("  Win Rate:        {:.2}%", stats.win_rate);
    println!("  Total Costs:     ${:.2}", stats.total_costs);
    println!("\nRisk Metrics:");
    println!("  Max Drawdown:    {:.2}%", stats.max_drawdown);
    println!("  Sharpe Ratio:    {:.4}", stats.sharpe_ratio);
    
    // Visualize signals
    let chart_path = config.output.output_dir.join("signal_chart.png");
    if let Err(e) = visualise_signals(&result, &chart_path) {
        eprintln!("Failed to create chart: {}", e);
    } else {
        println!("\nSignal chart saved to: {}", chart_path.display());
    }
}
