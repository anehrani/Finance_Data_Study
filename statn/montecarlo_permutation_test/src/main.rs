mod random;
mod file_io;
mod mcpt_bars;
mod mcpt_trend;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mcpt")]
#[command(about = "Monte Carlo Permutation Test for trading systems", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Mean reversion system using bar data (OHLC)
    Bars {
        /// Long-term rise lookback
        #[arg(value_name = "LOOKBACK")]
        lookback: usize,
        
        /// Number of MCPT replications (hundreds or thousands)
        #[arg(value_name = "NREPS")]
        nreps: usize,
        
        /// Market file (YYYYMMDD Open High Low Close)
        #[arg(value_name = "FILENAME")]
        filename: PathBuf,
    },
    
    /// Moving average crossover system
    Trend {
        /// Maximum moving-average lookback
        #[arg(value_name = "MAX_LOOKBACK")]
        max_lookback: usize,
        
        /// Number of MCPT replications (hundreds or thousands)
        #[arg(value_name = "NREPS")]
        nreps: usize,
        
        /// Market file (YYYYMMDD Price)
        #[arg(value_name = "FILENAME")]
        filename: PathBuf,
    },
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Bars { lookback, nreps, filename } => {
            println!("\nReading market file...");
            let data = file_io::read_ohlc_file(&filename)
                .map_err(|e| format!("Error reading file: {}", e))?;
            
            mcpt_bars::run_mcpt_bars(
                lookback,
                nreps,
                data.open,
                data.high,
                data.low,
                data.close,
            )
        }
        
        Commands::Trend { max_lookback, nreps, filename } => {
            println!("\nReading market file...");
            let prices = file_io::read_price_file(&filename)
                .map_err(|e| format!("Error reading file: {}", e))?;
            
            mcpt_trend::run_mcpt_trend(max_lookback, nreps, prices)
        }
    }
}
