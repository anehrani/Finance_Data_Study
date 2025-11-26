//! Backtesting module for simulating trading strategies.
//! 
//! This module now uses the common backtesting library from statn/src/backtesting
//! while maintaining backward compatibility with the existing API.

use crate::signals_generators::SignalResult;

/// Statistics from backtesting a trading strategy.
#[derive(Debug, Clone)]
pub struct TradeStats {
    /// Initial budget at start of trading.
    pub initial_budget: f64,
    /// Final budget after all trades.
    pub final_budget: f64,
    /// Total profit/loss (final - initial).
    pub total_pnl: f64,
    /// Return on investment as a percentage.
    pub roi_percent: f64,
    /// Total number of trades executed.
    pub num_trades: usize,
    /// Number of winning trades.
    pub num_wins: usize,
    /// Number of losing trades.
    pub num_losses: usize,
    /// Win rate as a percentage.
    pub win_rate: f64,
    /// Total transaction costs paid.
    pub total_costs: f64,
    /// Maximum drawdown experienced.
    pub max_drawdown: f64,
    /// Sharpe ratio (if applicable).
    pub sharpe_ratio: f64,
    /// History of budget over time.
    pub budget_history: Vec<f64>,
    /// History of positions (1 = long, -1 = short, 0 = flat).
    pub position_history: Vec<i32>,
    /// Detailed log of all trades.
    pub trades: Vec<TradeLog>,
}

/// Detailed information about a single trade.
#[derive(Debug, Clone)]
pub struct TradeLog {
    /// Index where the trade was opened.
    pub entry_index: usize,
    /// Price at which the trade was opened.
    pub entry_price: f64,
    /// Index where the trade was closed.
    pub exit_index: usize,
    /// Price at which the trade was closed.
    pub exit_price: f64,
    /// Type of trade: "LONG" or "SHORT".
    pub trade_type: String,
    /// Profit/Loss for this trade.
    pub pnl: f64,
    /// Return percentage for this trade.
    pub return_pct: f64,
}

/// Backtest a trading strategy based on generated signals.
///
/// This function now uses the common backtesting library while maintaining
/// the same API and behavior as the original implementation.
///
/// # Arguments
/// * `result` - The signal result containing prices and signals
/// * `initial_budget` - Starting capital for trading
/// * `transaction_cost_pct` - Transaction cost as a percentage (e.g., 0.1 for 0.1%)
///
/// # Returns
/// TradeStats with comprehensive trading statistics
pub fn backtest_signals(
    result: &SignalResult,
    initial_budget: f64,
    transaction_cost_pct: f64,
) -> TradeStats {
    // Create backtesting configuration
    let config = backtesting::BacktestConfig {
        initial_capital: initial_budget,
        transaction_cost: transaction_cost_pct / 100.0, // Convert from percentage to fraction
    };
    
    // Run backtest using the common library
    // Note: prices in result are in log space
    let backtest_result = backtesting::run_backtest_discrete(
        &result.prices,
        &result.signals,
        &config,
        true, // prices_in_log_space = true
    ).expect("Backtesting failed");
    
    // Convert BacktestResult to TradeStats for backward compatibility
    let final_budget = backtest_result.equity_curve.last().copied().unwrap_or(initial_budget);
    let total_pnl = final_budget - initial_budget;
    let roi_percent = backtest_result.metrics.get("ROI %").copied().unwrap_or(0.0);
    let num_trades = backtest_result.trades;
    let num_wins = backtest_result.metrics.get("Winning Trades").copied().unwrap_or(0.0) as usize;
    let num_losses = backtest_result.metrics.get("Losing Trades").copied().unwrap_or(0.0) as usize;
    let win_rate = backtest_result.metrics.get("Win Rate %").copied().unwrap_or(0.0);
    let total_costs = backtest_result.metrics.get("Total Costs").copied().unwrap_or(0.0);
    let max_drawdown = backtest_result.metrics.get("Max Drawdown %").copied().unwrap_or(0.0);
    let sharpe_ratio = backtest_result.metrics.get("Sharpe Ratio").copied().unwrap_or(0.0);
    
    // Convert trade logs
    let trades = backtest_result.trade_log
        .unwrap_or_default()
        .into_iter()
        .map(|t| TradeLog {
            entry_index: t.entry_index,
            entry_price: t.entry_price,
            exit_index: t.exit_index,
            exit_price: t.exit_price,
            trade_type: t.trade_type,
            pnl: t.pnl,
            return_pct: t.return_pct,
        })
        .collect();
    
    TradeStats {
        initial_budget,
        final_budget,
        total_pnl,
        roi_percent,
        num_trades,
        num_wins,
        num_losses,
        win_rate,
        total_costs,
        max_drawdown,
        sharpe_ratio,
        budget_history: backtest_result.equity_curve,
        position_history: backtest_result.position_history.unwrap_or_default(),
        trades,
    }
}
