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

/// Detailed information about a single trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeLog {
    /// Index where the trade was opened
    pub entry_index: usize,
    /// Price at which the trade was opened
    pub entry_price: f64,
    /// Index where the trade was closed
    pub exit_index: usize,
    /// Price at which the trade was closed
    pub exit_price: f64,
    /// Type of trade: "LONG" or "SHORT"
    pub trade_type: String,
    /// Profit/Loss for this trade
    pub pnl: f64,
    /// Return percentage for this trade
    pub return_pct: f64,
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
    /// Number of trades executed
    pub trades: usize,
    /// Detailed trade log (if enabled)
    pub trade_log: Option<Vec<TradeLog>>,
    /// Position history (1 = long, -1 = short, 0 = flat)
    pub position_history: Option<Vec<i32>>,
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
        trade_log: None,
        position_history: None,
    })
}

/// Run a backtest with discrete signals (1 = BUY, -1 = SELL, 0 = HOLD)
/// 
/// This function is designed for strategies that generate discrete trading signals
/// rather than continuous position sizing. It tracks detailed trade information
/// including entry/exit prices and P&L for each trade.
/// 
/// # Arguments
/// * `prices` - Price data (can be in log space or regular space)
/// * `signals` - Discrete signals: 1 = BUY, -1 = SELL, 0 = HOLD
/// * `config` - Backtesting configuration
/// * `prices_in_log_space` - Whether prices are in log space (will be converted to actual prices)
/// 
/// # Returns
/// BacktestResult with detailed trade logs and position history
pub fn run_backtest_discrete(
    prices: &[f64],
    signals: &[i32],
    config: &BacktestConfig,
    prices_in_log_space: bool,
) -> Result<BacktestResult> {
    let n = prices.len();
    if n == 0 {
        anyhow::bail!("No price data provided");
    }
    if signals.len() != n {
        anyhow::bail!("Signals and prices must have the same length");
    }

    let mut budget = config.initial_capital;
    let mut position: i32 = 0; // 0 = flat, 1 = long, -1 = short
    let mut entry_price = 0.0;
    let mut num_trades = 0;
    let mut num_wins = 0;
    let mut num_losses = 0;
    let mut total_costs = 0.0;
    let mut peak_budget = config.initial_capital;
    let mut max_drawdown = 0.0;
    
    let mut budget_history = Vec::with_capacity(n);
    let mut position_history = Vec::with_capacity(n);
    let mut returns = Vec::new();
    let mut trades = Vec::new();
    
    // Track trade entry details
    let mut current_entry_idx = 0;
    
    // Transaction cost as percentage (config uses fraction, convert to percentage)
    let transaction_cost_pct = config.transaction_cost;

    for i in 0..n {
        let price = if prices_in_log_space {
            prices[i].exp() // Convert from log space to actual price
        } else {
            prices[i]
        };
        let signal = signals[i];
        
        // Record current state
        budget_history.push(budget);
        position_history.push(position);
        
        // Process signal
        match (position, signal) {
            // Currently flat, got BUY signal -> go long
            (0, 1) => {
                let cost = budget * transaction_cost_pct;
                total_costs += cost;
                budget -= cost;
                entry_price = price;
                current_entry_idx = i;
                position = 1;
                num_trades += 1;
            }
            // Currently flat, got SELL signal -> go short
            (0, -1) => {
                let cost = budget * transaction_cost_pct;
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
                let cost = budget * transaction_cost_pct;
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
                let cost2 = budget * transaction_cost_pct;
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
                let cost = budget * transaction_cost_pct;
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
                let cost2 = budget * transaction_cost_pct;
                total_costs += cost2;
                budget -= cost2;
                entry_price = price;
                current_entry_idx = i;
                position = 1;
                num_trades += 2;
            }
            // Currently long, got HOLD or BUY -> update unrealized P&L
            (1, 0) | (1, 1) => {
                // Mark-to-market (unrealized)
                let unrealized_pnl = budget * (price / entry_price - 1.0);
                let current_value = budget + unrealized_pnl;
                budget_history[i] = current_value;
            }
            // Currently short, got HOLD or SELL -> update unrealized P&L
            (-1, 0) | (-1, -1) => {
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
        let final_price = if prices_in_log_space {
            prices[n - 1].exp()
        } else {
            prices[n - 1]
        };
        let pnl = if position == 1 {
            budget * (final_price / entry_price - 1.0)
        } else {
            budget * (entry_price / final_price - 1.0)
        };
        let cost = budget * transaction_cost_pct;
        budget += pnl - cost;
        total_costs += cost;
        
        // Update the last point in history to reflect the realized value (minus exit cost)
        if let Some(last) = budget_history.last_mut() {
            *last = budget;
        }
        
        if pnl > 0.0 {
            num_wins += 1;
        } else {
            num_losses += 1;
        }
        returns.push(pnl / budget);
        
        trades.push(TradeLog {
            entry_index: current_entry_idx,
            entry_price,
            exit_index: n - 1,
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
    
    let total_pnl = budget - config.initial_capital;
    let roi_percent = (total_pnl / config.initial_capital) * 100.0;
    let win_rate = if num_trades > 0 {
        (num_wins as f64 / (num_wins + num_losses) as f64) * 100.0
    } else {
        0.0
    };
    
    // Calculate daily returns from budget history
    let mut daily_returns = Vec::with_capacity(n - 1);
    for i in 0..n - 1 {
        let ret = if budget_history[i] > 0.0 {
            (budget_history[i + 1] - budget_history[i]) / budget_history[i]
        } else {
            0.0
        };
        daily_returns.push(ret);
    }

    // Calculate Sharpe ratio (annualized, assuming daily data) using daily returns
    let sharpe_ratio = if !daily_returns.is_empty() {
        let mean_return = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
        let variance = daily_returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / daily_returns.len() as f64;
        let std_dev = variance.sqrt();
        if std_dev > 1e-9 {
            (mean_return / std_dev) * (252.0_f64).sqrt() // Annualized
        } else {
            0.0
        }
    } else {
        0.0
    };
    
    // Build metrics
    let mut metrics = FxHashMap::default();
    metrics.insert("Total Return".to_string(), total_pnl / config.initial_capital);
    metrics.insert("ROI %".to_string(), roi_percent);
    metrics.insert("Total Trades".to_string(), num_trades as f64);
    metrics.insert("Winning Trades".to_string(), num_wins as f64);
    metrics.insert("Losing Trades".to_string(), num_losses as f64);
    metrics.insert("Win Rate %".to_string(), win_rate);
    metrics.insert("Total Costs".to_string(), total_costs);
    metrics.insert("Max Drawdown %".to_string(), max_drawdown * 100.0);
    metrics.insert("Sharpe Ratio".to_string(), sharpe_ratio);
    
    Ok(BacktestResult {
        equity_curve: budget_history,
        daily_returns,
        metrics,
        trades: num_trades,
        trade_log: Some(trades),
        position_history: Some(position_history),
    })
}
