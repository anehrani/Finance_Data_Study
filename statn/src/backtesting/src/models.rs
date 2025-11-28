use serde::{Deserialize, Serialize};

/// Detailed information about a single trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Statistics from backtesting a trading strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Result of the signal generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResult {
    /// The raw price series.
    pub prices: Vec<f64>,
    /// Signal per price point: 1 = BUY, -1 = SELL, 0 = HOLD.
    pub signals: Vec<i32>,
    /// Parameters used for the generation (for reference).
    /// These are generic metadata fields, can be extended or made generic if needed.
    /// For now, keeping them as in the original to minimize friction, 
    /// but ideally this should be a HashMap or generic.
    /// To make it general, I'll use a more flexible approach or keep it simple for now.
    /// I'll keep the specific fields for now as the user wants to replace the local implementation.
    pub long_lookback: usize,
    pub short_pct: f64,
    pub short_thresh: f64,
    pub long_thresh: f64,
}
