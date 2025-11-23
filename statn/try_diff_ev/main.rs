use std::process;

use statn::estimators::sensitivity::sensitivity;
use statn::estimators::StocBias;
use statn::models::differential_evolution::diff_ev;

use try_diff_ev::{
    backtest_signals, criter, generate_signals, load_market_data, load_parameters,
    save_parameters, visualise_signals, MarketData,
};

// Include entrypoint helper module
#[path = "entrypoint_helper.rs"]
mod entrypoint_helper;

use clap::Parser;
use entrypoint_helper::{Cli, Commands};






fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Optimize {
            data_file,
            max_lookback,
            max_thresh,
            popsize,
            max_gens,
            min_trades,
            output,
            output_dir,
            verbose,
        } => {
            println!("\n=== OPTIMIZATION MODE ===");
            println!("Data file: {}", data_file.display());
            println!("Max lookback: {}", max_lookback);
            println!("Output: {}\n", output_dir.join(&output).display());
            
            // Load market data
            let market_data = match load_market_data(&data_file, max_lookback) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            };
            println!("Loaded {} prices\n", market_data.prices.len());
            
            // Create output directory
            if let Err(e) = std::fs::create_dir_all(&output_dir) {
                eprintln!("Error creating output directory: {}", e);
                process::exit(1);
            }
            
            // Run optimization
            let low_bounds = vec![2.0, 0.01, 0.0, 0.0];
            let high_bounds = vec![max_lookback as f64, 99.0, max_thresh, max_thresh];
            
            let mut stoc_bias_opt = StocBias::new(market_data.prices.len() - max_lookback);
            if stoc_bias_opt.is_none() {
                eprintln!("Insufficient memory for StocBias");
                process::exit(1);
            }
            
            let sb_ptr = stoc_bias_opt.as_mut().unwrap() as *mut StocBias;
            let criter_wrapper = |params: &[f64], mintrades: i32| -> f64 {
                unsafe {
                    let mut sb_ref = Some(&mut *sb_ptr);
                    criter(params, mintrades, &market_data, &mut sb_ref)
                }
            };
            
            println!("Running differential evolution...");
            let result = diff_ev(
                criter_wrapper,
                4, 1, 100, max_gens, min_trades, 10000000,
                popsize, 0.2, 0.2, 0.3,
                &low_bounds, &high_bounds, verbose,
                &mut stoc_bias_opt,
            );
            
            match result {
                Ok(params) => {
                    println!("\n=== RESULTS ===");
                    println!("Best performance: {:.4}", params[4]);
                    println!("\nOptimal parameters:");
                    println!("  Long lookback:  {:.4}", params[0]);
                    println!("  Short %:        {:.4}", params[1]);
                    println!("  Short thresh:   {:.4}", params[2]);
                    println!("  Long thresh:    {:.4}", params[3]);
                    
                    if let Some(ref sb) = stoc_bias_opt {
                        let (is_mean, oos_mean, bias) = sb.compute();
                        println!("\nBias estimates:");
                        println!("  In-sample:      {:.4}", is_mean);
                        println!("  Out-of-sample:  {:.4}", oos_mean);
                        println!("  Bias:           {:.4}", bias);
                        println!("  Expected:       {:.4}", params[4] - bias);
                    }
                    
                    // Save parameters
                    let output_path = output_dir.join(&output);
                    if let Err(e) = save_parameters(&output_path, &params[0..4]) {
                        eprintln!("Error saving parameters: {}", e);
                    } else {
                        println!("\n✓ Parameters saved to: {}", output_path.display());
                    }
                    
                    // Sensitivity analysis
                    println!("\nRunning sensitivity analysis...");
                    let _ = sensitivity(
                        |p, m| criter(p, m, &market_data, &mut None),
                        4, 1, 30, 80, min_trades, &params,
                        &low_bounds, &high_bounds,
                    );
                    println!("✓ Sensitivity saved to SENS.LOG");
                }
                Err(e) => {
                    eprintln!("Optimization error: {}", e);
                    process::exit(1);
                }
            }
        }
        
        Commands::Predict {
            data_file,
            params_file,
            budget,
            transaction_cost,
            output_dir,
            verbose,
        } => {
            println!("\n=== PREDICTION MODE ===");
            println!("Data file: {}", data_file.display());
            println!("Parameters: {}", params_file.display());
            println!("Budget: ${:.2}\n", budget);
            
            // Load parameters
            let params = match load_parameters(&params_file) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error loading parameters: {}", e);
                    process::exit(1);
                }
            };
            
            if params.len() < 4 {
                eprintln!("Parameters file must contain at least 4 values");
                process::exit(1);
            }
            
            println!("Parameters:");
            println!("  Long lookback:  {:.4}", params[0]);
            println!("  Short %:        {:.4}", params[1]);
            println!("  Short thresh:   {:.4}", params[2]);
            println!("  Long thresh:    {:.4}\n", params[3]);
            
            // Load market data (use a reasonable max_lookback)
            let max_lookback = (params[0] as usize).max(100);
            let market_data = match load_market_data(&data_file, max_lookback) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            };
            println!("Loaded {} prices\n", market_data.prices.len());
            
            // Create output directory
            if let Err(e) = std::fs::create_dir_all(&output_dir) {
                eprintln!("Error creating output directory: {}", e);
                process::exit(1);
            }
            
            // Generate signals
            let result = generate_signals(
                &market_data.prices,
                (params[0] + 1.0e-10) as usize,
                params[1], params[2], params[3],
            );
            
            // Print last 20 signals
            if verbose {
                println!("Last 20 signals:");
                let start = result.signals.len().saturating_sub(20);
                for i in start..result.signals.len() {
                    let sig = match result.signals[i] {
                        1 => "BUY", -1 => "SELL", _ => "HOLD",
                    };
                    println!("{:>5}: price={:.4} -> {}", i, result.prices[i], sig);
                }
                println!();
            }
            
            // Backtest
            let stats = backtest_signals(&result, budget, transaction_cost);
            
            println!("=== BACKTEST RESULTS ===");
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
            
            // Visualize
            let chart_path = output_dir.join("signal_chart.png");
            if let Err(e) = visualise_signals(&result, &chart_path) {
                eprintln!("Failed to create chart: {}", e);
            } else {
                println!("\n✓ Chart saved to: {}", chart_path.display());
            }
        }
    }
    
    println!("\n✓ Completed successfully!");
}
