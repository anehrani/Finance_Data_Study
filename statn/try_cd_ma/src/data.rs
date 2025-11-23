use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Failed to open file: {path}")]
    FileOpen {
        path: String,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Invalid price data at line {line}: {reason}")]
    InvalidPrice { line: usize, reason: String },
    
    #[error("Insufficient data: need at least {needed} prices, got {got}")]
    InsufficientData { needed: usize, got: usize },
}

/// Market data structure
#[derive(Debug, Clone)]
pub struct MarketData {
    /// Log prices
    pub prices: Vec<f64>,
}

/// Training and test data split
#[derive(Debug)]
pub struct DataSplit {
    pub train_prices: Vec<f64>,
    pub test_prices: Vec<f64>,
    pub max_lookback: usize,
}

/// Load market prices from file (YYYYMMDD Price format)
pub fn load_prices<P: AsRef<Path>>(path: P) -> Result<Vec<f64>> {
    let path_str = path.as_ref().display().to_string();
    let file = File::open(&path).map_err(|e| DataError::FileOpen {
        path: path_str.clone(),
        source: e,
    })?;
    
    let reader = BufReader::new(file);
    
    let prices: Result<Vec<f64>> = reader
        .lines()
        .enumerate()
        .filter_map(|(line_num, line_result)| {
            match line_result {
                Ok(line) if line.trim().is_empty() => None,
                Ok(line) => Some(parse_price_line(&line, line_num + 1)),
                Err(e) => Some(Err(e.into())),
            }
        })
        .collect();
    
    let prices = prices?;
    
    if prices.is_empty() {
        anyhow::bail!("No valid price data found in file: {}", path_str);
    }
    
    println!("Loaded {} prices from {}", prices.len(), path_str);
    Ok(prices)
}

/// Parse a single line of price data
fn parse_price_line(line: &str, line_num: usize) -> Result<f64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return Err(DataError::InvalidPrice {
            line: line_num,
            reason: "Expected YYYYMMDD Price format".to_string(),
        }
        .into());
    }
    
    // Validate date format (8 digits)
    if parts[0].len() != 8 || !parts[0].chars().all(|c| c.is_ascii_digit()) {
        return Err(DataError::InvalidPrice {
            line: line_num,
            reason: format!("Invalid date format: {}", parts[0]),
        }
        .into());
    }
    
    // Parse price
    let price: f64 = parts[1].parse().map_err(|_| DataError::InvalidPrice {
        line: line_num,
        reason: format!("Invalid price: {}", parts[1]),
    })?;
    
    if price <= 0.0 {
        return Err(DataError::InvalidPrice {
            line: line_num,
            reason: format!("Price must be positive, got {}", price),
        }
        .into());
    }
    
    // Convert to log price
    Ok(price.ln())
}

/// Split data into training and test sets
pub fn split_train_test(
    prices: &[f64],
    max_lookback: usize,
    n_test: usize,
) -> Result<DataSplit> {
    let total_needed = max_lookback + n_test + 1;
    
    if prices.len() < total_needed {
        return Err(DataError::InsufficientData {
            needed: total_needed,
            got: prices.len(),
        }
        .into());
    }
    
    let n_train = prices.len() - n_test - max_lookback;
    
    // Training data: from start to (start + max_lookback + n_train)
    let train_end = max_lookback + n_train;
    let train_prices = prices[..train_end].to_vec();
    
    // Test data: from (train_end - max_lookback) to end
    // This ensures we have enough lookback for test indicators
    let test_start = train_end - max_lookback;
    let test_prices = prices[test_start..].to_vec();
    
    println!("Training cases: {}", n_train);
    println!("Test cases: {}", n_test);
    
    Ok(DataSplit {
        train_prices,
        test_prices,
        max_lookback,
    })
}

/// Compute target returns from prices
pub fn compute_targets(prices: &[f64], start_idx: usize, n_cases: usize) -> Vec<f64> {
    (0..n_cases)
        .map(|i| {
            let idx = start_idx + i;
            prices[idx + 1] - prices[idx]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_parse_price_line() {
        assert!(parse_price_line("20200101 100.0", 1).is_ok());
        assert!(parse_price_line("invalid", 1).is_err());
        assert!(parse_price_line("20200101 -10.0", 1).is_err());
        assert!(parse_price_line("20200101", 1).is_err());
    }
    
    #[test]
    fn test_load_prices() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "20200101 100.0").unwrap();
        writeln!(file, "20200102 101.5").unwrap();
        writeln!(file, "20200103 99.8").unwrap();
        
        let prices = load_prices(file.path()).unwrap();
        assert_eq!(prices.len(), 3);
        assert!((prices[0] - 100.0_f64.ln()).abs() < 1e-10);
    }
    
    #[test]
    fn test_split_train_test() {
        let prices: Vec<f64> = (0..1000).map(|i| (100.0 + i as f64).ln()).collect();
        let split = split_train_test(&prices, 200, 252).unwrap();
        
        assert_eq!(split.max_lookback, 200);
        assert!(split.train_prices.len() > 0);
        assert!(split.test_prices.len() >= 252 + 200);
    }
    
    #[test]
    fn test_compute_targets() {
        let prices = vec![1.0, 1.1, 1.05, 1.15];
        let targets = compute_targets(&prices, 0, 3);
        
        assert_eq!(targets.len(), 3);
        assert!((targets[0] - 0.1).abs() < 1e-10);
        assert!((targets[1] - (-0.05)).abs() < 1e-10);
        assert!((targets[2] - 0.1).abs() < 1e-10);
    }
}
