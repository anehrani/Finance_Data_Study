use anyhow::Result;
use clap::Parser;
use try_cd_ma::*;

fn main() -> Result<()> {
    println!("CD_MA - Moving Average Crossover Indicator Selection\n");
    
    // Load configuration
    let config = Config::parse();
    config.validate()?;
    
    // Load market data
    println!("Loading market data...");
    let prices = load_prices(std::path::Path::new(&config.data_file))
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Split into training and test sets
    let split = split_train_test(&prices, config.max_lookback(), config.n_test)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    println!("Training cases: {}", split.train_data.len() - split.max_lookback);
    println!("Test cases: {}", split.test_data.len() - split.max_lookback);
    
    // Generate indicator specifications
    let specs = generate_specs(
        config.lookback_inc,
        config.n_long,
        config.n_short,
    );
    println!("MA indicators: {}", config.n_ma_vars());

    println!("Total indicators: {}", specs.len());
    
    // Compute training indicators
    let n_train = split.train_data.len() - split.max_lookback - 1;
    
    // Validate sufficient training data
    if n_train < config.n_vars() + 10 {
        anyhow::bail!(
            "Insufficient training data: need at least {} cases, got {}",
            config.n_vars() + 10,
            n_train
        );
    }
    
    println!("Computing training indicators...");
    let train_data = compute_indicator_data(
        &split.train_data,
        split.max_lookback,
        n_train,
        &specs,
    )?;
    
    // Train model with cross-validation
    let training_result = train_with_cv(
        config.n_vars(),
        n_train,
        &train_data.data,
        &train_data.targets,
        config.alpha,
        config.n_folds,
        config.n_lambdas,
        config.max_iterations,
        config.tolerance,
    )?;
    
    // Compute test indicators and targets
    println!("Computing test indicators...");
    let test_data = compute_indicator_data(
        &split.test_data,
        split.max_lookback,
        config.n_test,
        &specs,
    )?;
    
    // Evaluate model
    let evaluation_result = evaluate_model(
        &training_result.model,
        &test_data.data,
        &test_data.targets,
        config.n_vars(),
    )?;
    
    // Run backtest on test data
    println!("\n{}", "=".repeat(60));
    println!("Running Backtest");
    println!("{}", "=".repeat(60));
    
    // Convert log prices to actual prices for backtesting
    let test_prices_actual: Vec<f64> = split.test_data
        .iter()
        .skip(split.max_lookback)
        .take(config.n_test)
        .map(|&log_price| log_price.exp())
        .collect();
    
    let initial_capital = 100_000.0;
    let transaction_cost = 0.1; // 0.1% transaction cost
    
    let backtest_result = try_cd_ma::run_backtest(
        &training_result.model,
        &test_prices_actual,
        &test_data.data,
        config.n_vars(),
        initial_capital,
        transaction_cost,
    )?;
    
    // Write backtest results
    let backtest_output = format!("{}backtest_results.txt", config.output_path);
    try_cd_ma::write_backtest_results(&backtest_output, &backtest_result)?;
    
    // Write results

    // Note: Model saving removed due to serialization requirements
    
    // Write results
    let results_path = format!("{}CD_MA.LOG", config.output_path);
    write_results(
        &results_path,
        &config,
        &training_result,
        &evaluation_result,
        &specs,
    )?;
    
    // Print summary
    println!("\n{}", "=".repeat(60));
    println!("Summary");
    println!("{}", "=".repeat(60));
    println!("\nModel Performance:");
    println!(
        "  In-sample explained variance: {:.3}%",
        100.0 * evaluation_result.in_sample_explained
    );
    println!(
        "  OOS total return: {:.5} ({:.3}%)",
        evaluation_result.oos_return, evaluation_result.oos_return_pct
    );
    
    println!("\nBacktest Performance:");
    println!(
        "  Total return: {:.2}%",
        backtest_result.roi_percent
    );
    println!(
        "  Total trades: {}",
        backtest_result.num_trades
    );
    println!(
        "  Win rate: {:.2}%",
        backtest_result.win_rate
    );
    println!(
        "  Max drawdown: {:.2}%",
        backtest_result.max_drawdown
    );
    println!(
        "  Sharpe ratio: {:.3}",
        backtest_result.sharpe_ratio
    );
    
    Ok(())
}