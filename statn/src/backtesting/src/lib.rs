use anyhow::Result;
use serde::{Serialize, Deserialize};
use rustc_hash::FxHashMap;

pub mod metrics;
pub mod report;

pub use metrics::calculate_metrics;
pub use report::{generate_text_report, generate_json_report};

/// Configuration for backtesting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// Initial capital
    pub initial_capital: f64,
    /// Transaction cost per trade (as a fraction, e.g., 0.001 for 0.1%)
    pub transaction_cost: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100_000.0,
            transaction_cost: 0.0,
        }
    }
}

/// Result of a backtest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    /// Equity curve (daily values)
    pub equity_curve: Vec<f64>,
    /// Daily returns
    pub daily_returns: Vec<f64>,
    /// Performance metrics
    pub metrics: FxHashMap<String, f64>,
    /// Trade log (optional, could be added later)
    pub trades: usize,
}

/// Trait for trading strategies
pub trait Strategy {
    /// Generate a signal for a given time step
    /// Returns a value between -1.0 (short) and 1.0 (long)
    /// 0.0 means neutral/cash
    fn signal(&self, data: &[f64], index: usize) -> f64;
}

/// Run a backtest
pub fn run_backtest<S: Strategy>(
    strategy: &S,
    prices: &[f64],
    config: &BacktestConfig,
) -> Result<BacktestResult> {
    let n = prices.len();
    if n == 0 {
        anyhow::bail!("No price data provided");
    }

    let mut equity = vec![config.initial_capital; n];
    let mut cash = config.initial_capital;
    let mut position = 0.0; // Number of shares/units
    let mut trades = 0;

    for i in 0..n - 1 {
        let current_price = prices[i];
        let signal = strategy.signal(prices, i);
        
        // Simple execution: target position = signal * (total_equity / price)
        // This assumes signal is % of equity to allocate
        let total_equity = cash + position * current_price;
        let target_value = signal * total_equity;
        let target_position = target_value / current_price;
        
        let trade_size = target_position - position;
        
        if trade_size.abs() > 1e-6 {
            let trade_value = trade_size * current_price;
            let cost = trade_value.abs() * config.transaction_cost;
            
            cash -= trade_value + cost;
            position += trade_size;
            trades += 1;
        }
        
        // Update equity for next step (mark to market)
        equity[i + 1] = cash + position * prices[i + 1];
    }

    // Calculate daily returns
    let mut daily_returns = Vec::with_capacity(n - 1);
    for i in 0..n - 1 {
        let ret = if equity[i] > 0.0 {
            (equity[i + 1] - equity[i]) / equity[i]
        } else {
            0.0
        };
        daily_returns.push(ret);
    }

    // Calculate metrics (placeholder for now)
    let mut metrics = FxHashMap::default();
    let total_return = (equity.last().unwrap() - config.initial_capital) / config.initial_capital;
    metrics.insert("Total Return".to_string(), total_return);
    metrics.insert("Trades".to_string(), trades as f64);

    Ok(BacktestResult {
        equity_curve: equity,
        daily_returns,
        metrics,
        trades,
    })
}
