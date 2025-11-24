use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use try_cd_ma::{Config, load_prices, generate_specs, compute_indicator_data, CDMAStrategy};
use statn::models::cd_ma::CoordinateDescent;
use backtesting::{BacktestConfig, run_backtest, generate_text_report};

/// Command-line arguments for backtesting
#[derive(Parser, Debug)]
#[command(name = "backtest")]
#[command(about = "Backtest CD_MA model on unseen data")]
struct Args {
    /// Path to TOML configuration file (for indicator specs)
    #[arg(short, long)]
    config: PathBuf,

    /// Path to saved model file (JSON)
    #[arg(short, long)]
    model: PathBuf,

    /// Path to unseen market data file
    #[arg(short, long)]
    data: PathBuf,

    /// Path to output report file
    #[arg(short, long)]
    output: PathBuf,

    /// Initial capital
    #[arg(long, default_value = "100000.0")]
    initial_capital: f64,

    /// Transaction cost (fraction)
    #[arg(long, default_value = "0.0")]
    transaction_cost: f64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("CD_MA Backtesting (using shared backtesting package)\n");

    // Load configuration
    println!("Loading configuration from {}...", args.config.display());
    let config = Config::from_file(&args.config)?;

    // Load model
    println!("Loading model from {}...", args.model.display());
    let model_file = std::fs::File::open(&args.model)
        .with_context(|| format!("Failed to open model file: {}", args.model.display()))?;
    let model: CoordinateDescent = serde_json::from_reader(model_file)
        .with_context(|| "Failed to parse model file")?;

    // Load market data
    println!("Loading market data from {}...", args.data.display());
    let prices = load_prices(&args.data)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Generate indicator specifications
    let specs = generate_specs(
        config.lookback_inc,
        config.n_long,
        config.n_short,
        config.enable_rsi,
        &config.rsi_periods,
    );
    println!("MA indicators: {}", config.n_ma_vars());
    if config.enable_rsi {
        println!("RSI indicators: {}", config.n_rsi_vars());
    }
    println!("Total indicators: {}", specs.len());

    // Compute indicators for the new data
    // We use the entire dataset for testing here
    let n_cases = prices.len() - config.max_lookback();
    if n_cases <= 0 {
        anyhow::bail!("Insufficient data for backtesting");
    }

    println!("Computing indicators for {} cases...", n_cases);
    let test_data = compute_indicator_data(
        &prices,
        config.max_lookback(),
        n_cases,
        &specs,
    )?;

    // Create strategy
    // Note: prices vector contains all prices.
    // test_data.data contains indicators for prices[max_lookback..]
    // So the offset is max_lookback.
    let strategy = CDMAStrategy::new(
        model,
        test_data.data,
        config.n_vars(),
        config.max_lookback(),
    );

    // Run backtest
    let backtest_config = BacktestConfig {
        initial_capital: args.initial_capital,
        transaction_cost: args.transaction_cost,
    };

    println!("Running backtest...");
    // We pass the raw prices (converted to non-log if needed, but here we assume log prices are OK for signal generation,
    // but for equity calculation we might want real prices.
    // The current backtesting package assumes prices are tradeable prices.
    // try_cd_ma uses log prices.
    // Let's convert log prices back to real prices for the backtester.
    let real_prices: Vec<f64> = prices.iter().map(|&p| p.exp()).collect();
    
    let result = run_backtest(&strategy, &real_prices, &backtest_config)?;

    // Write report
    println!("Writing report to {}...", args.output.display());
    generate_text_report(&result, &args.output)?;
    
    println!("\nBacktest completed successfully.");
    println!("Total Return: {:.4}", result.metrics.get("Total Return").unwrap_or(&0.0));
    println!("Sharpe Ratio: {:.4}", result.metrics.get("Sharpe Ratio").unwrap_or(&0.0));

    Ok(())
}
