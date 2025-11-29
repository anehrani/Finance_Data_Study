pub mod config;
pub mod data;
pub mod indicators;
pub mod training;
pub mod evaluation;
pub mod backtest;

pub use config::Config;
pub use data::{load_prices, split_train_test};
pub use indicators::{generate_specs, compute_indicator_data};
pub use training::train_with_cv;
pub use evaluation::{evaluate_model, write_results, write_backtest_results};
pub use backtest::{generate_signals, run_backtest};
