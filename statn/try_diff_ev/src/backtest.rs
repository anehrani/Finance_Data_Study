//! Backtesting module for simulating trading strategies.
//! This module now delegates to the general `backtesting` library.

pub use backtesting::{backtest_signals, TradeLog, TradeStats};
