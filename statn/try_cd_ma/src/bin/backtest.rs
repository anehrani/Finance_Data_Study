use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use try_cd_ma::{Config, load_prices, generate_specs, compute_indicator_data, evaluate_model};
use statn::models::cd_ma::CoordinateDescent;

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
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("CD_MA Backtesting\n");

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
    let specs = generate_specs(config.lookback_inc, config.n_long, config.n_short);
    println!("Number of indicators: {}", specs.len());

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

    // Evaluate model
    let evaluation_result = evaluate_model(
        &model,
        &test_data.data,
        &test_data.targets,
        config.n_vars(),
    )?;

    // Write report
    println!("Writing report to {}...", args.output.display());
    let mut file = std::fs::File::create(&args.output)?;
    use std::io::Write;
    
    writeln!(file, "CD_MA Backtest Report")?;
    writeln!(file, "=====================")?;
    writeln!(file)?;
    writeln!(file, "Configuration: {}", args.config.display())?;
    writeln!(file, "Model: {}", args.model.display())?;
    writeln!(file, "Data: {}", args.data.display())?;
    writeln!(file)?;
    writeln!(file, "Results:")?;
    writeln!(file, "  Total cases: {}", n_cases)?;
    writeln!(file, "  Total return: {:.5}", evaluation_result.oos_return)?;
    writeln!(file, "  Return percentage: {:.3}%", evaluation_result.oos_return_pct)?;
    
    println!("\nBacktest completed successfully.");
    println!("Total return: {:.5} ({:.3}%)", evaluation_result.oos_return, evaluation_result.oos_return_pct);

    Ok(())
}
