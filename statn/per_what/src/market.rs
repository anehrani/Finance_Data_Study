use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use anyhow::{Context, Result};

/// Reads market prices from a file.
/// Expected format: YYYYMMDD Price
/// Returns a vector of log prices.
pub fn read_market_prices(filename: &str) -> Result<Vec<f64>> {
    let path = Path::new(filename);
    let file = File::open(path).with_context(|| format!("Cannot open market history file {}", filename))?;
    let reader = BufReader::new(file);
    let mut prices = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("Error reading line {} of file {}", line_num + 1, filename))?;
        let line = line.trim();

        if line.is_empty() || line.len() < 2 {
            continue;
        }

        // Parse date (simple check)
        if line.len() < 8 || !line[..8].chars().all(|c| c.is_ascii_digit()) {
             return Err(anyhow::anyhow!("Invalid date reading line {} of file {}", line_num + 1, filename));
        }

        // Parse price
        // Find the start of the price (skip date and delimiters)
        let price_str = line[8..].trim_start_matches(|c| c == ' ' || c == '\t' || c == ',');
        
        if let Ok(price) = price_str.parse::<f64>() {
            if price > 0.0 {
                prices.push(price.ln());
            } else {
                 // Handle non-positive prices if necessary, for now just skip or error? 
                 // The C++ code does: if (prices[nprices] > 0.0) prices[nprices] = log ( prices[nprices] ) ;
                 // It implies it might store non-log price if <= 0, but then it uses it in sums. 
                 // Let's assume valid prices are positive.
                 prices.push(price); // Push raw if not positive? Or just 0? C++ logic is a bit loose there.
                 // Actually, log(<=0) is undefined. 
                 // Let's stick to the C++ logic: if > 0, log it. If not, keep it as is (likely 0 or negative).
            }
        } else {
             // If parsing fails, maybe it's not a critical error if we want to be robust, 
             // but C++ exits on error. Let's error.
             return Err(anyhow::anyhow!("Invalid price reading line {} of file {}", line_num + 1, filename));
        }
    }

    Ok(prices)
}
