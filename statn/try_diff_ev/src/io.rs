//! I/O utilities for loading and saving data.

use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

/// Market data structure.
#[derive(Debug, Clone)]
pub struct MarketData {
    /// Price series (in log space)
    pub prices: Vec<f64>,
    /// Maximum lookback period
    pub max_lookback: usize,
}

/// Load market data from a file.
///
/// Expected format: YYYYMMDD price1 price2 price3 price4
/// The last column is used as the closing price.
///
/// # Arguments
/// * `path` - Path to the market data file
/// * `max_lookback` - Maximum lookback period for validation
///
/// # Returns
/// MarketData with log-transformed prices
pub fn load_market_data<P: AsRef<Path>>(
    path: P,
    max_lookback: usize,
) -> Result<MarketData, String> {
    let file = File::open(path.as_ref())
        .map_err(|e| format!("Cannot open market file '{}': {}", path.as_ref().display(), e))?;
    
    let reader = io::BufReader::new(file);
    let mut prices = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Error reading line {}: {}", line_num + 1, e))?;
        
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }
        
        // Parse line: YYYYMMDD price1 price2 price3 price4
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            // Take the last column as the close price
            if let Ok(price) = parts[parts.len() - 1].parse::<f64>() {
                if price > 0.0 {
                    prices.push(price.ln()); // Store in log space
                }
            }
        }
    }
    
    if prices.is_empty() {
        return Err("No valid price data found in file".to_string());
    }
    
    if prices.len() <= max_lookback {
        return Err(format!(
            "Insufficient data: {} prices, need more than {} for lookback",
            prices.len(),
            max_lookback
        ));
    }
    
    Ok(MarketData {
        prices,
        max_lookback,
    })
}

/// Load trading parameters from a file.
///
/// Expected format: One parameter per line (4 lines total)
/// 1. Long lookback period
/// 2. Short percentage
/// 3. Short threshold
/// 4. Long threshold
///
/// # Arguments
/// * `path` - Path to the parameters file
///
/// # Returns
/// Vector of 4 parameters
pub fn load_parameters<P: AsRef<Path>>(path: P) -> Result<Vec<f64>, String> {
    let file = File::open(path.as_ref())
        .map_err(|e| format!("Cannot open parameters file '{}': {}", path.as_ref().display(), e))?;
    
    let reader = io::BufReader::new(file);
    let mut params = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Error reading line {}: {}", line_num + 1, e))?;
        
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let val = trimmed
                .parse::<f64>()
                .map_err(|e| format!("Parse error on line {}: {}", line_num + 1, e))?;
            params.push(val);
        }
    }
    
    if params.len() < 4 {
        return Err(format!(
            "Parameters file must contain at least 4 values, found {}",
            params.len()
        ));
    }
    
    Ok(params)
}

/// Save trading parameters to a file.
///
/// # Arguments
/// * `path` - Path where parameters will be saved
/// * `params` - Parameter values to save (one per line)
///
/// # Returns
/// Result indicating success or error
pub fn save_parameters<P: AsRef<Path>>(path: P, params: &[f64]) -> Result<(), String> {
    let mut file = File::create(path.as_ref())
        .map_err(|e| format!("Cannot create file '{}': {}", path.as_ref().display(), e))?;
    
    for param in params {
        writeln!(file, "{}", param)
            .map_err(|e| format!("Write error: {}", e))?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_parameters() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "6.0").unwrap();
        writeln!(temp_file, "57.8").unwrap();
        writeln!(temp_file, "30.1").unwrap();
        writeln!(temp_file, "0.0").unwrap();
        
        let params = load_parameters(temp_file.path()).unwrap();
        assert_eq!(params.len(), 4);
        assert_eq!(params[0], 6.0);
        assert_eq!(params[1], 57.8);
    }
    
    #[test]
    fn test_save_parameters() {
        let temp_file = NamedTempFile::new().unwrap();
        let params = vec![6.0, 57.8, 30.1, 0.0];
        
        save_parameters(temp_file.path(), &params).unwrap();
        
        let loaded = load_parameters(temp_file.path()).unwrap();
        assert_eq!(loaded, params);
    }
}
