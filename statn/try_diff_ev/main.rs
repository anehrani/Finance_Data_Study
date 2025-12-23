use std::fs::File;
use std::io::Write;
use std::process;

use statn::estimators::sensitivity::sensitivity;
use statn::estimators::StocBias;
use statn::models::differential_evolution::diff_ev;

use try_diff_ev::{
    backtest_signals, criter, criter_enhanced, generate_signals,
    load_market_data, load_parameters, save_parameters, visualise_signals, MarketData,
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
            train_pct,
            params_file,
            sensitivity_log,
            generator,
            output_dir,
            verbose,
        } => {
            println!("\n=== OPTIMIZATION MODE ===");
            println!("Data file: {}", data_file.display());
            println!("Max lookback: {}", max_lookback);
            println!("Output: {}\n", output_dir.join(&params_file).display());
            
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
            let split_idx = (market_data.prices.len() as f64 * train_pct) as usize;
            if split_idx < max_lookback + 10 {
                eprintln!("Training set too small: {} prices", split_idx);
                process::exit(1);
            }
            
            println!("Training on first {} prices ({:.1}%)", split_idx, train_pct * 100.0);
            
            // Create training market data
            let train_data = MarketData {
                prices: market_data.prices[..split_idx].to_vec(),
                max_lookback: market_data.max_lookback,
            };
            
            let low_bounds = vec![2.0, 0.01, 0.0, 0.0];
            let high_bounds = vec![max_lookback as f64, 99.0, max_thresh, max_thresh];
            
            let mut stoc_bias_opt = StocBias::new(train_data.prices.len() - max_lookback);
            if stoc_bias_opt.is_none() {
                eprintln!("Insufficient memory for StocBias");
                process::exit(1);
            }
            
            let sb_ptr = stoc_bias_opt.as_mut().unwrap() as *mut StocBias;
            let criter_wrapper = |params: &[f64], mintrades: i32| -> f64 {
                unsafe {
                    let mut sb_ref = Some(&mut *sb_ptr);
                    match generator.as_str() {
                        "log_diff" | "enhanced" => criter_enhanced(params, mintrades, &train_data, &mut sb_ref),
                        _ => criter(params, mintrades, &train_data, &mut sb_ref),
                    }
                }
            };
            
            
            println!("Running differential evolution...");
            
            let config = statn::models::differential_evolution::DiffEvConfig {
                nvars: 4,
                nints: 1,
                popsize: 100,
                overinit: max_gens,
                mintrades: min_trades,
                max_evals: 10000000,
                max_bad_gen: popsize,
                mutate_dev: 0.2,
                pcross: 0.2,
                pclimb: 0.3,
                low_bounds: &low_bounds,
                high_bounds: &high_bounds,
                print_progress: verbose,
            };
            
            let result = diff_ev(
                criter_wrapper,
                config,
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
                    let output_path = output_dir.join(&params_file);
                    if let Err(e) = save_parameters(&output_path, &params[0..4]) {
                        eprintln!("Error saving parameters: {}", e);
                    } else {
                        println!("\n✓ Parameters saved to: {}", output_path.display());
                    }
                    
                    // Sensitivity analysis
                    println!("\nRunning sensitivity analysis...");
                    
                    let sens_config = statn::estimators::sensitivity::SensitivityConfig {
                        nvars: 4,
                        nints: 1,
                        npoints: 30,
                        nres: 80,
                        mintrades: min_trades,
                        best: &params,
                        low_bounds: &low_bounds,
                        high_bounds: &high_bounds,
                    };
                    
                    let _ = sensitivity(
                        |p, m| match generator.as_str() {
                            "log_diff" | "enhanced" => criter_enhanced(p, m, &train_data, &mut None),
                            _ => criter(p, m, &train_data, &mut None),
                        },
                        4, 1, 30, 80, min_trades, &params,
                        &low_bounds, &high_bounds,
                        &output_dir.join(&sensitivity_log),
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
            train_pct,
            output_dir,
            generator,
            verbose,
        } => {
            println!("\n=== PREDICTION MODE ===");
            println!("Data file: {}", data_file.display());
            println!("Parameters: {}", params_file.display());
            println!("Budget: ${:.2}\n", budget);
            
            // Load parameters
            let params = match load_parameters(&output_dir.join(params_file)) {
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
            println!("Using signal generator: {}", generator);
            let result = generate_signals(
                &generator,
                &market_data.prices,
                (params[0] + 1.0e-10) as usize,
                params[1], params[2], params[3],
            );
            
            // Slice for backtesting (unseen data)
            let split_idx = (market_data.prices.len() as f64 * train_pct) as usize;
            println!("Backtesting on unseen data: prices {} to {} ({:.1}% of data)", 
                     split_idx, market_data.prices.len(), (1.0 - train_pct) * 100.0);
            
            if split_idx >= result.prices.len() {
                eprintln!("No data left for backtesting!");
                process::exit(1);
            }
            
            // Create result slice for backtesting
            // We need to construct a new SignalResult with the sliced data
            let test_result = try_diff_ev::SignalResult {
                prices: result.prices[split_idx..].to_vec(),
                signals: result.signals[split_idx..].to_vec(),
                long_lookback: result.long_lookback,
                short_pct: result.short_pct,
                short_thresh: result.short_thresh,
                long_thresh: result.long_thresh,
            };
            
            // Print last 20 signals of the TEST set
            if verbose {
                println!("Last 20 signals (of test set):");
                let start = test_result.signals.len().saturating_sub(20);
                for i in start..test_result.signals.len() {
                    let sig = match test_result.signals[i] {
                        1 => "BUY", -1 => "SELL", _ => "HOLD",
                    };
                    println!("{:>5}: price={:.4} -> {}", i + split_idx, test_result.prices[i], sig);
                }
                println!();
            }
            
            // Backtest
            let stats = backtest_signals(&test_result, budget, transaction_cost);
            
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
            
            // Write trade log to file
            let log_path = output_dir.join("trade_log.txt");
            match File::create(&log_path) {
                Ok(mut file) => {
                    writeln!(file, "=== TRADE LOG ===").unwrap();
                    writeln!(file, "{:<5} {:<8} {:<10} {:<10} {:<10} {:<10} {:<8}", 
                             "Type", "Entry Idx", "Entry Price", "Exit Idx", "Exit Price", "P&L", "Return").unwrap();
                    writeln!(file, "{}", "-".repeat(70)).unwrap();
                    
                    for trade in &stats.trades {
                        writeln!(file, "{:<5} {:<8} {:<10.4} {:<10} {:<10.4} {:<10.2} {:>7.2}%",
                                 trade.trade_type,
                                 trade.entry_index + split_idx,
                                 trade.entry_price,
                                 trade.exit_index + split_idx,
                                 trade.exit_price,
                                 trade.pnl,
                                 trade.return_pct).unwrap();
                    }
                    println!("\n✓ Trade log saved to: {}", log_path.display());
                }
                Err(e) => eprintln!("Failed to write trade log: {}", e),
            }

            // Print detailed trade log if verbose
            if verbose {
                println!("\n=== TRADE LOG ===");
                println!("{:<5} {:<8} {:<10} {:<10} {:<10} {:<10} {:<8}", 
                         "Type", "Entry Idx", "Entry Price", "Exit Idx", "Exit Price", "P&L", "Return");
                println!("{}", "-".repeat(70));
                
                for trade in &stats.trades {
                    println!("{:<5} {:<8} {:<10.4} {:<10} {:<10.4} {:<10.2} {:>7.2}%",
                             trade.trade_type,
                             trade.entry_index + split_idx, // Adjust index to global
                             trade.entry_price,
                             trade.exit_index + split_idx, // Adjust index to global
                             trade.exit_price,
                             trade.pnl,
                             trade.return_pct);
                }
                println!("{}", "-".repeat(70));
            }

            // Visualize
            let chart_path = output_dir.join("signal_chart.png");
            if let Err(e) = visualise_signals(&test_result, Some(&stats), &chart_path) {
                eprintln!("Failed to create chart: {}", e);
            } else {
                println!("\n✓ Chart saved to: {}", chart_path.display());
            }
        }
    }
    
    println!("\n✓ Completed successfully!");
}
