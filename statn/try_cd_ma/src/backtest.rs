use anyhow::Result;
use backtesting::{BacktestConfig, BacktestResult, run_backtest_discrete};
use statn::models::cd_ma::CoordinateDescent;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// Run backtesting on test data using the trained model
/// 
/// # Arguments
/// * `model` - Trained coordinate descent model
/// * `test_prices` - Test price data (in regular space, not log)
/// * `test_data` - Pre-computed indicator data (standardized)
/// * `n_vars` - Number of variables per case
/// * `config` - Backtesting configuration
/// 
/// # Returns
/// BacktestResult with detailed trade logs and metrics
pub fn run_backtest(
    model: &CoordinateDescent,
    test_prices: &[f64],
    test_data: &[f64],
    n_vars: usize,
    config: &BacktestConfig,
) -> Result<BacktestResult> {
    println!("\nRunning backtest on test data...");
    
    let n_test = test_prices.len();
    
    // Generate signals for each test case
    let mut signals = Vec::with_capacity(n_test);
    
    for i in 0..n_test {
        // Check if we have indicator data for this index
        if (i + 1) * n_vars > test_data.len() {
            // No data available yet, hold
            signals.push(0);
            continue;
        }
        
        let xptr = &test_data[i * n_vars..(i + 1) * n_vars];
        
        // Compute prediction
        let pred: f64 = xptr
            .iter()
            .enumerate()
            .map(|(ivar, &x)| {
                model.beta[ivar] * (x - model.xmeans[ivar]) / model.xscales[ivar]
            })
            .sum();
        
        let pred = pred * model.yscale + model.ymean;
        
        // Trading logic: long if pred > 0, short if pred < 0
        let signal = if pred > 0.0 {
            1
        } else if pred < 0.0 {
            -1
        } else {
            0
        };
        
        signals.push(signal);
    }
    
    // Run discrete backtest
    let result = run_backtest_discrete(
        test_prices,
        &signals,
        config,
        false, // prices are not in log space
    )?;
    
    println!("Backtest completed:");
    println!("  Total trades: {}", result.trades);
    println!("  Total return: {:.2}%", result.metrics.get("ROI %").unwrap_or(&0.0));
    println!("  Win rate: {:.2}%", result.metrics.get("Win Rate %").unwrap_or(&0.0));
    println!("  Max drawdown: {:.2}%", result.metrics.get("Max Drawdown %").unwrap_or(&0.0));
    println!("  Sharpe ratio: {:.3}", result.metrics.get("Sharpe Ratio").unwrap_or(&0.0));
    
    Ok(result)
}

/// Write backtest results to file
pub fn write_backtest_results<P: AsRef<Path>>(
    path: P,
    result: &BacktestResult,
) -> Result<()> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path.as_ref())?;
    
    writeln!(file, "Backtest Results")?;
    writeln!(file, "{}", "=".repeat(60))?;
    writeln!(file)?;
    
    // Performance metrics
    writeln!(file, "Performance Metrics:")?;
    writeln!(file, "  Total Return: {:.2}%", result.metrics.get("ROI %").unwrap_or(&0.0))?;
    writeln!(file, "  Total Trades: {}", result.trades)?;
    writeln!(file, "  Winning Trades: {}", *result.metrics.get("Winning Trades").unwrap_or(&0.0) as usize)?;
    writeln!(file, "  Losing Trades: {}", *result.metrics.get("Losing Trades").unwrap_or(&0.0) as usize)?;
    writeln!(file, "  Win Rate: {:.2}%", result.metrics.get("Win Rate %").unwrap_or(&0.0))?;
    writeln!(file, "  Max Drawdown: {:.2}%", result.metrics.get("Max Drawdown %").unwrap_or(&0.0))?;
    writeln!(file, "  Sharpe Ratio: {:.3}", result.metrics.get("Sharpe Ratio").unwrap_or(&0.0))?;
    writeln!(file, "  Total Costs: ${:.2}", result.metrics.get("Total Costs").unwrap_or(&0.0))?;
    writeln!(file)?;
    
    // Trade log
    if let Some(trades) = &result.trade_log {
        writeln!(file, "Trade Log:")?;
        writeln!(file, "  {:<8} {:<12} {:<8} {:<12} {:<8} {:<12} {:<10}", 
                 "Type", "Entry Idx", "Entry $", "Exit Idx", "Exit $", "P&L", "Return %")?;
        writeln!(file, "  {}", "-".repeat(80))?;
        
        for trade in trades {
            writeln!(
                file,
                "  {:<8} {:<12} ${:<11.2} {:<8} ${:<11.2} ${:<11.2} {:>9.2}%",
                trade.trade_type,
                trade.entry_index,
                trade.entry_price,
                trade.exit_index,
                trade.exit_price,
                trade.pnl,
                trade.return_pct
            )?;
        }
        writeln!(file)?;
    }
    
    // Equity curve summary
    writeln!(file, "Equity Curve Summary:")?;
    writeln!(file, "  Initial Capital: ${:.2}", result.equity_curve.first().unwrap_or(&0.0))?;
    writeln!(file, "  Final Capital: ${:.2}", result.equity_curve.last().unwrap_or(&0.0))?;
    writeln!(file)?;
    
    println!("Backtest results written to {}", path.as_ref().display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use statn::models::cd_ma::CoordinateDescent;
    
    #[test]
    fn test_run_backtest() {
        let n_vars = 3;
        let n_cases = 10;
        let mut model = CoordinateDescent::new(n_vars, n_cases, false, true, 0);
        
        // Set up dummy model parameters
        model.beta = vec![0.1, 0.2, -0.1];
        model.xmeans = vec![0.0; n_vars];
        model.xscales = vec![1.0; n_vars];
        model.ymean = 0.0;
        model.yscale = 1.0;
        model.explained = 0.5;
        
        let test_prices = vec![100.0, 101.0, 102.0, 101.5, 103.0, 102.0, 104.0, 105.0, 104.5, 106.0];
        let test_data = vec![0.0; n_vars * n_cases];
        
        let config = BacktestConfig {
            initial_capital: 100_000.0,
            transaction_cost: 0.001,
        };
        
        let result = run_backtest(&model, &test_prices, &test_data, n_vars, &config);
        assert!(result.is_ok());
    }
}
