use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for CD_MA analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Increment to long-term lookback
    pub lookback_inc: usize,
    
    /// Number of long-term lookbacks to test
    pub n_long: usize,
    
    /// Number of short-term lookbacks to test
    pub n_short: usize,

    /// Crossover types to generate (e.g., ["ma", "rsi"])
    #[serde(default = "default_crossover_types")]
    pub crossover_types: Vec<crate::indicators::CrossoverType>,


    
    /// Alpha parameter for elastic net (0-1]
    pub alpha: f64,
    
    /// Path to market data file (YYYYMMDD Price format)
    pub data_file: PathBuf,
    
    /// Path to output results file
    #[serde(default = "default_output_file")]
    pub output_file: PathBuf,
    
    /// Number of test cases (default: 252 = one year)
    #[serde(default = "default_n_test")]
    pub n_test: usize,
    
    /// Number of cross-validation folds
    #[serde(default = "default_n_folds")]
    pub n_folds: usize,
    
    /// Number of lambda values to test
    #[serde(default = "default_n_lambdas")]
    pub n_lambdas: usize,
    
    /// Maximum iterations for coordinate descent
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,
    
    /// Convergence tolerance
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
}

fn default_output_file() -> PathBuf {
    PathBuf::from("CD_MA.LOG")
}

fn default_n_test() -> usize {
    252
}

fn default_n_folds() -> usize {
    10
}

fn default_n_lambdas() -> usize {
    50
}

fn default_max_iterations() -> usize {
    1000
}

fn default_tolerance() -> f64 {
    1e-9
}

fn default_crossover_types() -> Vec<crate::indicators::CrossoverType> {
    vec![crate::indicators::CrossoverType::Ma]
}

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(name = "try_cd_ma")]
#[command(about = "Moving Average Crossover Indicator Selection using Coordinate Descent")]
pub struct Args {
    /// Path to TOML configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    
    /// Increment to long-term lookback
    #[arg(value_name = "LOOKBACK_INC")]
    pub lookback_inc: Option<usize>,
    
    /// Number of long-term lookbacks
    #[arg(value_name = "N_LONG")]
    pub n_long: Option<usize>,
    
    /// Number of short-term lookbacks
    #[arg(value_name = "N_SHORT")]
    pub n_short: Option<usize>,

    /// Crossover types (comma-separated: ma, rsi)
    #[arg(long, value_delimiter = ',')]
    pub crossover_types: Option<Vec<String>>,


    
    /// Alpha parameter (0-1]
    #[arg(value_name = "ALPHA")]
    pub alpha: Option<f64>,
    
    /// Market data file
    #[arg(value_name = "FILENAME")]
    pub filename: Option<PathBuf>,
}

impl Config {
    /// Load configuration from TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Create configuration from command-line arguments
    pub fn from_args(args: &Args) -> Result<Self> {
        let config = Config {
            lookback_inc: args.lookback_inc
                .ok_or_else(|| anyhow::anyhow!("lookback_inc is required"))?,
            n_long: args.n_long
                .ok_or_else(|| anyhow::anyhow!("n_long is required"))?,
            n_short: args.n_short
                .ok_or_else(|| anyhow::anyhow!("n_short is required"))?,
            crossover_types: if let Some(types) = &args.crossover_types {
                types.iter().map(|s| match s.to_lowercase().as_str() {
                    "ma" => Ok(crate::indicators::CrossoverType::Ma),
                    "rsi" => Ok(crate::indicators::CrossoverType::Rsi),
                    "ema" => Ok(crate::indicators::CrossoverType::Ema),
                    "macd" => Ok(crate::indicators::CrossoverType::Macd),
                    "roc" => Ok(crate::indicators::CrossoverType::Roc),
                    _ => Err(anyhow::anyhow!("Unknown crossover type: {}", s)),
                }).collect::<Result<Vec<_>>>()?
            } else {
                default_crossover_types()
            },
            alpha: args.alpha
                .ok_or_else(|| anyhow::anyhow!("alpha is required"))?,
            data_file: args.filename.clone()
                .ok_or_else(|| anyhow::anyhow!("filename is required"))?,
            output_file: default_output_file(),
            n_test: default_n_test(),
            n_folds: default_n_folds(),
            n_lambdas: default_n_lambdas(),
            max_iterations: default_max_iterations(),
            tolerance: default_tolerance(),
        };
        
        config.validate()?;
        Ok(config)
    }
    
    /// Load configuration from either file or command-line arguments
    pub fn load() -> Result<Self> {
        let args = Args::parse();
        
        if let Some(config_path) = &args.config {
            Self::from_file(config_path)
        } else {
            Self::from_args(&args)
        }
    }
    
    /// Validate configuration parameters
    pub fn validate(&self) -> Result<()> {
        if self.alpha <= 0.0 || self.alpha > 1.0 {
            anyhow::bail!("Alpha must be in range (0, 1], got {}", self.alpha);
        }
        
        if self.lookback_inc == 0 {
            anyhow::bail!("lookback_inc must be greater than 0");
        }
        
        if self.n_long == 0 {
            anyhow::bail!("n_long must be greater than 0");
        }
        
        if self.n_short == 0 {
            anyhow::bail!("n_short must be greater than 0");
        }
        
        if self.n_test == 0 {
            anyhow::bail!("n_test must be greater than 0");
        }
        
        if self.n_folds < 2 {
            anyhow::bail!("n_folds must be at least 2");
        }
        
        Ok(())
    }
    
    /// Get total number of indicator variables
    pub fn n_vars(&self) -> usize {
        self.n_long * self.n_short * self.crossover_types.len()
    }
    
    /// Get maximum lookback period
    pub fn max_lookback(&self) -> usize {
        let mut ma_max = if !self.crossover_types.is_empty() {
            self.n_long * self.lookback_inc
        } else {
            0
        };
        
        // If MACD crossover is used, we need extra lookback for the signal line (9)
        if self.crossover_types.contains(&crate::indicators::CrossoverType::Macd) {
            ma_max += 9;
        }

        ma_max
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_validation() {
        let mut config = Config {
            lookback_inc: 10,
            n_long: 20,
            n_short: 10,
            crossover_types: vec![crate::indicators::CrossoverType::Ma],

            alpha: 0.5,
            data_file: PathBuf::from("test.txt"),
            output_file: PathBuf::from("output.log"),
            n_test: 252,
            n_folds: 10,
            n_lambdas: 50,
            max_iterations: 1000,
            tolerance: 1e-9,
        };
        
        assert!(config.validate().is_ok());
        
        config.alpha = 1.5;
        assert!(config.validate().is_err());
        
        config.alpha = 0.0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_n_vars() {
        let config = Config {
            lookback_inc: 10,
            n_long: 20,
            n_short: 10,
            crossover_types: vec![crate::indicators::CrossoverType::Ma],

            alpha: 0.5,
            data_file: PathBuf::from("test.txt"),
            output_file: PathBuf::from("output.log"),
            n_test: 252,
            n_folds: 10,
            n_lambdas: 50,
            max_iterations: 1000,
            tolerance: 1e-9,
        };
        
        assert_eq!(config.n_vars(), 200);
        assert_eq!(config.max_lookback(), 200);
    }
}
