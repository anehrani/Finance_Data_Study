use anyhow::Result;
use try_cd_comb::*;

fn main() -> Result<()> {
    println!("CD_MA - Moving Average Crossover Indicator Selection\n");
    
    // Load configuration
    let config = Config::load()?;
    
    // Load market data
    println!("Loading market data...");
    let prices = load_prices(&config.data_file)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Split into training and test sets
    let split = split_train_test(&prices, config.max_lookback(), config.n_test)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    println!("Training cases: {}", split.train_data.len() - split.max_lookback);
    println!("Test cases: {}", split.test_data.len() - split.max_lookback);
    
    // Generate indicator specifications
    let specs = generate_specs(config.lookback_inc, config.n_long, config.n_short, &config.rsi_periods, &config.macd_configs, &config.crossover_types);
    println!("Number of indicators: {}", specs.len());
    
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
    
    // Write results
    write_results(
        &config.output_file,
        &config,
        &training_result,
        &evaluation_result,
        &specs,
    )?;
    
    // Run backtest
    println!("Running backtest...");
    // Extract test prices (log prices) corresponding to the test period
    let test_prices_slice = &split.test_data[split.max_lookback..split.max_lookback + config.n_test];
    
    let backtest_stats = run_backtest(
        &training_result.model,
        test_prices_slice,
        &test_data.data,
        config.n_test,
        config.n_vars(),
        10000.0, // Initial budget
        0.1,     // Transaction cost %
    )?;
    
    // Write backtest results
    let backtest_path = config.output_file.parent().unwrap_or(std::path::Path::new(".")).join("backtest_results.txt");
    write_backtest_results(&backtest_path, &backtest_stats)?;
    
    // Print summary
    println!("\nSummary:");
    println!(
        "  In-sample explained variance: {:.3}%",
        100.0 * evaluation_result.in_sample_explained
    );
    println!(
        "  OOS total return: {:.5} ({:.3}%)",
        evaluation_result.oos_return, evaluation_result.oos_return_pct
    );
    
    Ok(())
}
