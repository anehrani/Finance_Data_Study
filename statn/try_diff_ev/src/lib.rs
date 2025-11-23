//! Library utilities for the `try_diff_ev` package.
//!
//! This library provides a complete toolkit for trading signal generation,
//! backtesting, and visualization using moving-average crossover strategies.
//!
//! # Modules
//!
//! - `config` - Configuration structures using serde
//! - `io` - File I/O utilities for loading/saving data
//! - `signals` - Generate BUY/SELL/HOLD signals from price data
//! - `backtest` - Simulate trading with transaction costs and track performance
//! - `visualization` - Create charts showing price and trading signals

pub mod backtest;
pub mod config;
pub mod evaluators;
pub mod io;
pub mod signals_generators;
pub mod test_system;
pub mod test_system_enhanced;
pub mod visualization;

// Re-export commonly used types and functions
pub use backtest::{backtest_signals, TradeStats};
pub use config::Config;
pub use evaluators::{criter, criter_enhanced};
pub use io::{load_market_data, load_parameters, save_parameters, MarketData};
pub use signals_generators::{generate_signals, SignalResult};
pub use test_system_enhanced::test_system_enhanced;
pub use visualization::visualise_signals;
