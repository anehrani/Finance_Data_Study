use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Trading system using moving average crossover with differential evolution
#[derive(Parser, Debug)]
#[command(name = "try_diff_ev")]
#[command(about = "Trading signal generation, optimization, and backtesting", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run differential evolution optimization to find best parameters
    Optimize {
        /// Path to market data file
        #[arg(short, long)]
        data_file: PathBuf,
        
        /// Maximum lookback period
        #[arg(short = 'l', long, default_value_t = 6)]
        max_lookback: usize,
        
        /// Maximum threshold (×10000)
        #[arg(short = 't', long, default_value_t = 57.8112)]
        max_thresh: f64,
        
        /// Population size
        #[arg(short, long, default_value_t = 300)]
        popsize: usize,
        
        /// Maximum generations
        #[arg(short = 'g', long, default_value_t = 10000)]
        max_gens: usize,
        
        /// Minimum trades required
        #[arg(short = 'm', long, default_value_t = 20)]
        min_trades: i32,
        
        /// Output file for optimized parameters
        #[arg(short, long, default_value = "params.txt")]
        output: PathBuf,
        
        /// Output directory
        #[arg(short = 'D', long, default_value = "../results/")]
        output_dir: PathBuf,
        
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Generate signals and backtest using optimized parameters
    Predict {
        /// Path to market data file
        #[arg(short, long)]
        data_file: PathBuf,
        
        /// File containing optimized parameters
        #[arg(short, long, default_value = "params.txt")]
        params_file: PathBuf,
        
        /// Initial budget for backtesting
        #[arg(short, long, default_value_t = 10000.0)]
        budget: f64,
        
        /// Transaction cost percentage
        #[arg(short = 'c', long, default_value_t = 0.1)]
        transaction_cost: f64,
        
        /// Output directory
        #[arg(short = 'D', long, default_value = "../results/")]
        output_dir: PathBuf,
        
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}
