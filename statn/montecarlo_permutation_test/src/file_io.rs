use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct OhlcData {
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
}

/// Read market data file with OHLC format (YYYYMMDD Open High Low Close)
pub fn read_ohlc_file<P: AsRef<Path>>(filename: P) -> Result<OhlcData, String> {
    let file = File::open(filename.as_ref())
        .map_err(|e| format!("Cannot open market history file: {}", e))?;
    
    let reader = BufReader::new(file);
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
        
        // Parse prices
        let parts: Vec<&str> = line[8..]
            .split(|c: char| c == ' ' || c == '\t' || c == ',')
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
        
        // Convert to log prices
        if o > 0.0 && h > 0.0 && l > 0.0 && c > 0.0 {
            open.push(o.ln());
            high.push(h.ln());
            low.push(l.ln());
            close.push(c.ln());
        } else {
            return Err(format!("Non-positive price on line {}", line_num + 1));
        }
    }
    
    if open.is_empty() {
        return Err("No valid data found in file".to_string());
    }
    
    Ok(OhlcData { open, high, low, close })
}

/// Read market data file with single price format (YYYYMMDD Price)
pub fn read_price_file<P: AsRef<Path>>(filename: P) -> Result<Vec<f64>, String> {
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
            .split(|c: char| c == ' ' || c == '\t' || c == ',')
            .find(|s| !s.is_empty())
            .ok_or_else(|| format!("No price found on line {}", line_num + 1))?;
        
        let price = price_str.parse::<f64>()
            .map_err(|_| format!("Invalid price on line {}", line_num + 1))?;
        
        // Convert to log price
        if price > 0.0 {
            prices.push(price.ln());
        } else {
            return Err(format!("Non-positive price on line {}", line_num + 1));
        }
    }
    
    if prices.is_empty() {
        return Err("No valid data found in file".to_string());
    }
    
    Ok(prices)
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
    fn test_read_ohlc_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "20200101 100.0 102.0 99.0 101.0").unwrap();
        writeln!(file, "20200102 101.0 103.0 100.5 102.5").unwrap();
        
        let data = read_ohlc_file(file.path()).unwrap();
        assert_eq!(data.open.len(), 2);
        assert_eq!(data.high.len(), 2);
        assert_eq!(data.low.len(), 2);
        assert_eq!(data.close.len(), 2);
    }
}
