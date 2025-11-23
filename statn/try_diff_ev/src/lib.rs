//! Library utilities for the `try_diff_ev` package.
//!
//! This library provides a complete toolkit for trading signal generation,
//! backtesting, and visualization using moving-average crossover strategies.
//!
//! # Modules
//!
//! - `config` - Configuration structures using serde
//! - `signals` - Generate BUY/SELL/HOLD signals from price data
//! - `backtest` - Simulate trading with transaction costs and track performance
//! - `visualization` - Create charts showing price and trading signals

pub mod backtest;
pub mod config;
pub mod signals;
pub mod visualization;

// Re-export commonly used types and functions
pub use backtest::{backtest_signals, TradeStats};
pub use config::Config;
pub use signals::{generate_signals, SignalResult};
pub use visualization::visualise_signals;
