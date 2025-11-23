//! Backtesting module for simulating trading strategies.

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
/// Simulates trading with an initial budget, tracking positions, costs, and performance.
/// Note: prices should be in log space (as used in the system).
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
    let mut budget = initial_budget;
    let mut position: i32 = 0; // 0 = flat, 1 = long, -1 = short
    let mut entry_price = 0.0;
    let mut num_trades = 0;
    let mut num_wins = 0;
    let mut num_losses = 0;
    let mut total_costs = 0.0;
    let mut peak_budget = initial_budget;
    let mut max_drawdown = 0.0;
    
    let mut budget_history = Vec::with_capacity(result.prices.len());
    let mut position_history = Vec::with_capacity(result.prices.len());
    let mut returns = Vec::new();
    let mut trades = Vec::new();
    
    // Track trade entry details
    let mut current_entry_idx = 0;

    for i in 0..result.prices.len() {
        let price = result.prices[i].exp(); // Convert from log space to actual price
        let signal = result.signals[i];
        
        // Record current state
        budget_history.push(budget);
        position_history.push(position);
        
        // Process signal
        match (position, signal) {
            // Currently flat, got BUY signal -> go long
            (0, 1) => {
                let cost = budget * transaction_cost_pct / 100.0;
                total_costs += cost;
                budget -= cost;
                entry_price = price;
                current_entry_idx = i;
                position = 1;
                num_trades += 1;
            }
            // Currently flat, got SELL signal -> go short
            (0, -1) => {
                let cost = budget * transaction_cost_pct / 100.0;
                total_costs += cost;
                budget -= cost;
                entry_price = price;
                current_entry_idx = i;
                position = -1;
                num_trades += 1;
            }
            // Currently long, got SELL signal -> close long and go short
            (1, -1) => {
                // Close long position
                let pnl = budget * (price / entry_price - 1.0);
                let cost = budget * transaction_cost_pct / 100.0;
                budget += pnl - cost;
                total_costs += cost;
                
                if pnl > 0.0 {
                    num_wins += 1;
                } else {
                    num_losses += 1;
                }
                returns.push(pnl / budget);
                
                // Record trade
                trades.push(TradeLog {
                    entry_index: current_entry_idx,
                    entry_price,
                    exit_index: i,
                    exit_price: price,
                    trade_type: "LONG".to_string(),
                    pnl,
                    return_pct: (price / entry_price - 1.0) * 100.0,
                });

                // Open short position
                let cost2 = budget * transaction_cost_pct / 100.0;
                total_costs += cost2;
                budget -= cost2;
                entry_price = price;
                current_entry_idx = i;
                position = -1;
                num_trades += 2;
            }
            // Currently short, got BUY signal -> close short and go long
            (-1, 1) => {
                // Close short position
                let pnl = budget * (entry_price / price - 1.0);
                let cost = budget * transaction_cost_pct / 100.0;
                budget += pnl - cost;
                total_costs += cost;
                
                if pnl > 0.0 {
                    num_wins += 1;
                } else {
                    num_losses += 1;
                }
                returns.push(pnl / budget);
                
                // Record trade
                trades.push(TradeLog {
                    entry_index: current_entry_idx,
                    entry_price,
                    exit_index: i,
                    exit_price: price,
                    trade_type: "SHORT".to_string(),
                    pnl,
                    return_pct: (entry_price / price - 1.0) * 100.0,
                });

                // Open long position
                let cost2 = budget * transaction_cost_pct / 100.0;
                total_costs += cost2;
                budget -= cost2;
                entry_price = price;
                current_entry_idx = i;
                position = 1;
                num_trades += 2;
            }
            // Currently long, got HOLD -> update unrealized P&L
            (1, 0) => {
                // Mark-to-market (unrealized)
                let unrealized_pnl = budget * (price / entry_price - 1.0);
                let current_value = budget + unrealized_pnl;
                budget_history[i] = current_value;
            }
            // Currently short, got HOLD -> update unrealized P&L
            (-1, 0) => {
                // Mark-to-market (unrealized)
                let unrealized_pnl = budget * (entry_price / price - 1.0);
                let current_value = budget + unrealized_pnl;
                budget_history[i] = current_value;
            }
            _ => {} // No action needed
        }
        
        // Track drawdown
        if budget_history[i] > peak_budget {
            peak_budget = budget_history[i];
        }
        let drawdown = (peak_budget - budget_history[i]) / peak_budget;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }
    
    // Close any open position at the end
    if position != 0 {
        let final_price = result.prices[result.prices.len() - 1].exp();
        let pnl = if position == 1 {
            budget * (final_price / entry_price - 1.0)
        } else {
            budget * (entry_price / final_price - 1.0)
        };
        let cost = budget * transaction_cost_pct / 100.0;
        budget += pnl - cost;
        total_costs += cost;
        
        if pnl > 0.0 {
            num_wins += 1;
        } else {
            num_losses += 1;
        }
        returns.push(pnl / budget);
        
        trades.push(TradeLog {
            entry_index: current_entry_idx,
            entry_price,
            exit_index: result.prices.len() - 1,
            exit_price: final_price,
            trade_type: if position == 1 { "LONG".to_string() } else { "SHORT".to_string() },
            pnl,
            return_pct: if position == 1 { 
                (final_price / entry_price - 1.0) * 100.0 
            } else { 
                (entry_price / final_price - 1.0) * 100.0 
            },
        });
        
        num_trades += 1;
    }
    
    let total_pnl = budget - initial_budget;
    let roi_percent = (total_pnl / initial_budget) * 100.0;
    let win_rate = if num_trades > 0 {
        (num_wins as f64 / (num_wins + num_losses) as f64) * 100.0
    } else {
        0.0
    };
    
    // Calculate Sharpe ratio (annualized, assuming daily data)
    let sharpe_ratio = if !returns.is_empty() {
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();
        if std_dev > 0.0 {
            (mean_return / std_dev) * (252.0_f64).sqrt() // Annualized
        } else {
            0.0
        }
    } else {
        0.0
    };
    
    TradeStats {
        initial_budget,
        final_budget: budget,
        total_pnl,
        roi_percent,
        num_trades,
        num_wins,
        num_losses,
        win_rate,
        total_costs,
        max_drawdown: max_drawdown * 100.0, // Convert to percentage
        sharpe_ratio,
        budget_history,
        position_history,
        trades,
    }
}
