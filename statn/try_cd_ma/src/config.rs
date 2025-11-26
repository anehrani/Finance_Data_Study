use anyhow::Result;
use clap::Parser;
use serde::Deserialize;


/// Configuration for CD_MA analysis
#[derive(Debug, Clone, Deserialize, Parser)]
#[command(name = "try_cd_ma")]
#[command(about = "Moving Average Crossover Indicator Selection using Coordinate Descent")]
pub struct Config {

    /// Increment to long-term lookback
    #[arg(long, default_value_t = 6)]
    pub lookback_inc: usize,
    
    /// Number of long-term lookbacks to test
    #[arg(long, default_value_t = 6)]
    pub n_long: usize,
    
    /// Number of short-term lookbacks to test
    #[arg(long, default_value_t = 3)]
    pub n_short: usize,
    
    /// Alpha parameter for elastic net (0-1]
    #[arg(long, default_value_t = 0.5)]
    pub alpha: f64,
    
    /// Path to market data file (YYYYMMDD Price format)
    #[arg(value_name = "DATA_FILE")]
    pub data_file: String,
    
    /// Path to output results file
    #[arg(long, default_value = "results/")]
    pub output_path: String,
    
    /// Number of test cases (default: 252 = one year)
    #[arg(long, default_value_t = 100)]
    pub n_test: usize,
    
    /// Number of cross-validation folds
    #[arg(long, default_value_t = 10)]
    pub n_folds: usize,
    
    /// Number of lambda values to test
    #[arg(long, default_value_t = 50)]
    pub n_lambdas: usize,
    
    /// Maximum iterations for coordinate descent
    #[arg(long, default_value_t = 1000)]
    pub max_iterations: usize,
    
    /// Convergence tolerance
    #[arg(long, default_value_t = 1e-9)]
    pub tolerance: f64,
    
}

impl Config {
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

    /// Load configuration from TOML file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Get total number of indicator variables
    pub fn n_vars(&self) -> usize {
        self.n_long * self.n_short
    }
    
    /// Get number of MA indicator variables
    pub fn n_ma_vars(&self) -> usize {
        self.n_long * self.n_short
    }
    

    
    /// Get maximum lookback period
    pub fn max_lookback(&self) -> usize {
        self.n_long * self.lookback_inc
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
            alpha: 0.5,
            data_file: "test.txt".to_string(),
            output_path: "output.log".to_string(),
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
            alpha: 0.5,
            data_file: "test.txt".to_string(),
            output_path: "output.log".to_string(),
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
