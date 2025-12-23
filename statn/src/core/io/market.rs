use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// OHLC market data structure
#[derive(Debug, Clone)]
pub struct OhlcData {
    pub date: Vec<u32>,
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
}

impl OhlcData {
    /// Get the number of bars
    pub fn len(&self) -> usize {
        self.open.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.open.is_empty()
    }
}

/// Read market data file with single price format (YYYYMMDD Price)
/// Returns log prices by default
pub fn read_price_file<P: AsRef<Path>>(filename: P) -> Result<Vec<f64>, String> {
    read_price_file_impl(filename, true)
}

/// Read market data file with single price format (YYYYMMDD Price)
/// Returns raw prices (not log-transformed)
pub fn read_price_file_raw<P: AsRef<Path>>(filename: P) -> Result<Vec<f64>, String> {
    read_price_file_impl(filename, false)
}

/// Internal implementation for reading price files
fn read_price_file_impl<P: AsRef<Path>>(filename: P, use_log: bool) -> Result<Vec<f64>, String> {
    let file = File::open(filename.as_ref())
        .map_err(|e| format!("Cannot open market history file: {}", e))?;
    
    let reader = BufReader::new(file);
    let mut prices = Vec::new();
    
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result
            .map_err(|e| format!("Error reading line {}: {}", line_num + 1, e))?;
        
        if line.trim().is_empty() {
            continue;
        }
        
        // Parse the date (first 8 characters)
        if line.len() < 8 {
            return Err(format!("Line {} too short", line_num + 1));
        }
        
        let date_str = &line[..8];
        if !date_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("Invalid date on line {}", line_num + 1));
        }
        
        // Parse price
        let price_str = line[8..]
            .split([' ', '\t', ','])
            .find(|s| !s.is_empty())
            .ok_or_else(|| format!("No price found on line {}", line_num + 1))?;
        
        let price = price_str.parse::<f64>()
            .map_err(|_| format!("Invalid price on line {}", line_num + 1))?;
        
        if price <= 0.0 {
            return Err(format!("Non-positive price on line {}", line_num + 1));
        }
        
        // Convert to log price if requested
        prices.push(if use_log { price.ln() } else { price });
    }
    
    if prices.is_empty() {
        return Err("No valid data found in file".to_string());
    }
    
    Ok(prices)
}

/// Read market data file with OHLC format (YYYYMMDD Open High Low Close)
/// Returns log prices by default
pub fn read_ohlc_file<P: AsRef<Path>>(filename: P) -> Result<OhlcData, String> {
    read_ohlc_file_impl(filename, true)
}

/// Read market data file with OHLC format (YYYYMMDD Open High Low Close)
/// Returns raw prices (not log-transformed)
pub fn read_ohlc_file_raw<P: AsRef<Path>>(filename: P) -> Result<OhlcData, String> {
    read_ohlc_file_impl(filename, false)
}

/// Internal implementation for reading OHLC files
fn read_ohlc_file_impl<P: AsRef<Path>>(filename: P, use_log: bool) -> Result<OhlcData, String> {
    let file = File::open(filename.as_ref())
        .map_err(|e| format!("Cannot open market history file: {}", e))?;
    
    let reader = BufReader::new(file);
    let mut date = Vec::new();
    let mut open = Vec::new();
    let mut high = Vec::new();
    let mut low = Vec::new();
    let mut close = Vec::new();
    
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result
            .map_err(|e| format!("Error reading line {}: {}", line_num + 1, e))?;
        
        if line.trim().is_empty() {
            continue;
        }
        
        // Parse the date (first 8 characters)
        if line.len() < 8 {
            return Err(format!("Line {} too short", line_num + 1));
        }
        
        let date_str = &line[..8];
        if !date_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("Invalid date on line {}", line_num + 1));
        }
        
        let date_val = date_str.parse::<u32>()
            .map_err(|_| format!("Invalid date format on line {}", line_num + 1))?;

        // Parse prices
        let parts: Vec<&str> = line[8..]
            .split([' ', '\t', ','])
            .filter(|s| !s.is_empty())
            .collect();
        
        if parts.len() < 4 {
            return Err(format!("Insufficient price data on line {}", line_num + 1));
        }
        
        let o = parts[0].parse::<f64>()
            .map_err(|_| format!("Invalid open price on line {}", line_num + 1))?;
        let h = parts[1].parse::<f64>()
            .map_err(|_| format!("Invalid high price on line {}", line_num + 1))?;
        let l = parts[2].parse::<f64>()
            .map_err(|_| format!("Invalid low price on line {}", line_num + 1))?;
        let c = parts[3].parse::<f64>()
            .map_err(|_| format!("Invalid close price on line {}", line_num + 1))?;
        
        // Validate OHLC relationship
        if l > o || l > c || h < o || h < c {
            return Err(format!(
                "Invalid open/high/low/close relationship on line {}",
                line_num + 1
            ));
        }
        
        // Validate positive prices
        if o <= 0.0 || h <= 0.0 || l <= 0.0 || c <= 0.0 {
            return Err(format!("Non-positive price on line {}", line_num + 1));
        }
        
        // Convert to log prices if requested
        if use_log {
            open.push(o.ln());
            high.push(h.ln());
            low.push(l.ln());
            close.push(c.ln());
            open.push(o);
            high.push(h);
            low.push(l);
            close.push(c);
        }
        date.push(date_val);
    }
    
    if open.is_empty() {
        return Err("No valid data found in file".to_string());
    }
    
    Ok(OhlcData { date, open, high, low, close })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_read_price_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "20200101 100.0").unwrap();
        writeln!(file, "20200102 101.5").unwrap();
        writeln!(file, "20200103 99.8").unwrap();
        
        let prices = read_price_file(file.path()).unwrap();
        assert_eq!(prices.len(), 3);
        assert!((prices[0] - 100.0_f64.ln()).abs() < 1e-10);
    }
    
    #[test]
    fn test_read_price_file_raw() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "20200101 100.0").unwrap();
        writeln!(file, "20200102 101.5").unwrap();
        
        let prices = read_price_file_raw(file.path()).unwrap();
        assert_eq!(prices.len(), 2);
        assert!((prices[0] - 100.0).abs() < 1e-10);
        assert!((prices[1] - 101.5).abs() < 1e-10);
    }
    
    #[test]
    fn test_read_ohlc_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "20200101 100.0 102.0 99.0 101.0").unwrap();
        writeln!(file, "20200102 101.0 103.0 100.5 102.5").unwrap();
        
        let data = read_ohlc_file(file.path()).unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data.date[0], 20200101);
        assert!((data.open[0] - 100.0_f64.ln()).abs() < 1e-10);
    }
    
    #[test]
    fn test_read_ohlc_file_raw() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "20200101 100.0 102.0 99.0 101.0").unwrap();
        
        let data = read_ohlc_file_raw(file.path()).unwrap();
        assert_eq!(data.len(), 1);
        assert!((data.open[0] - 100.0).abs() < 1e-10);
        assert!((data.high[0] - 102.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_invalid_date() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid 100.0").unwrap();
        
        assert!(read_price_file(file.path()).is_err());
    }
    
    #[test]
    fn test_negative_price() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "20200101 -100.0").unwrap();
        
        assert!(read_price_file(file.path()).is_err());
    }
    
    #[test]
    fn test_invalid_ohlc_relationship() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "20200101 100.0 99.0 101.0 100.5").unwrap(); // high < low
        
        assert!(read_ohlc_file(file.path()).is_err());
    }
    
    #[test]
    fn test_empty_file() {
        let file = NamedTempFile::new().unwrap();
        assert!(read_price_file(file.path()).is_err());
    }
}
