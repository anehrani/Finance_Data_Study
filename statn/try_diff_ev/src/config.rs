//! Configuration structures for the trading system.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration for the trading system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Mode of operation: "optimize" or "predict"
    pub mode: String,
    
    /// Market data configuration
    pub market: MarketConfig,
    
    /// Optimization parameters (used in optimize mode)
    #[serde(default)]
    pub optimization: OptimizationConfig,
    
    /// Backtesting parameters (used in predict mode)
    #[serde(default)]
    pub backtest: BacktestConfig,
    
    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,
}

/// Market data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConfig {
    /// Path to the market data file (YYYYMMDD Price format)
    pub data_file: PathBuf,
    
    /// Maximum lookback period for moving averages
    pub max_lookback: usize,
    
    /// Maximum threshold value (×10000)
    pub max_thresh: f64,
}

/// Optimization configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Population size for differential evolution
    #[serde(default = "default_popsize")]
    pub popsize: usize,
    
    /// Maximum number of generations
    #[serde(default = "default_max_gens")]
    pub max_gens: usize,
    
    /// Minimum number of trades required
    #[serde(default = "default_min_trades")]
    pub min_trades: i32,
    
    /// Enable verbose output during optimization
    #[serde(default)]
    pub verbose: bool,
    
    /// File to save optimized parameters (optional)
    pub params_file: Option<PathBuf>,
}

/// Backtesting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// File containing optimized parameters
    pub params_file: PathBuf,
    
    /// Initial budget for trading
    #[serde(default = "default_initial_budget")]
    pub initial_budget: f64,
    
    /// Transaction cost as percentage (e.g., 0.1 for 0.1%)
    #[serde(default = "default_transaction_cost")]
    pub transaction_cost_pct: f64,
}

/// Output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Directory for output files (charts, logs, etc.)
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
    
    /// Enable verbose output
    #[serde(default)]
    pub verbose: bool,
}

// Default value functions
fn default_popsize() -> usize { 300 }
fn default_max_gens() -> usize { 10000 }
fn default_min_trades() -> i32 { 20 }
fn default_initial_budget() -> f64 { 10000.0 }
fn default_transaction_cost() -> f64 { 0.1 }
fn default_output_dir() -> PathBuf { PathBuf::from(".") }

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            popsize: default_popsize(),
            max_gens: default_max_gens(),
            min_trades: default_min_trades(),
            verbose: false,
            params_file: None,
        }
    }
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            params_file: PathBuf::from("params.txt"),
            initial_budget: default_initial_budget(),
            transaction_cost_pct: default_transaction_cost(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            verbose: false,
        }
    }
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to a TOML file.
    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
