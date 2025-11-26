use anyhow::{Context, Result, bail};
use std::fs::File;
use std::io::{BufRead, BufReader};

const BLOCK_SIZE: usize = 4096;

#[derive(Debug)]
pub struct MarketData {
    pub name: String,
    pub dates: Vec<i32>,
    pub close: Vec<f64>,
}

impl MarketData {
    fn new(name: String) -> Self {
        Self {
            name,
            dates: Vec::with_capacity(BLOCK_SIZE),
            close: Vec::with_capacity(BLOCK_SIZE),
        }
    }
}

/// Load markets from a file list
pub fn load_markets(file_list_path: &str) -> Result<Vec<MarketData>> {
    let file = File::open(file_list_path)
        .with_context(|| format!("Cannot open list file {}", file_list_path))?;
    let reader = BufReader::new(file);

    let mut markets = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        
        if line.is_empty() {
            continue;
        }

        // Extract market file name
        let market_file: String = line
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '\\' || *c == ':' || *c == '.' || *c == '/')
            .collect();

        if market_file.is_empty() {
            continue;
        }

        // Extract market name from file name (before last period)
        let market_name = extract_market_name(&market_file)?;

        println!("Reading market file {}...", market_file);

        let market_data = read_market_file(&market_file, &market_name)?;
        markets.push(market_data);
    }

    if markets.is_empty() {
        bail!("No markets loaded from file list");
    }

    Ok(markets)
}

fn extract_market_name(file_path: &str) -> Result<String> {
    // Find the last period
    let last_dot = file_path.rfind('.')
        .with_context(|| format!("Market file name ({}) has no extension", file_path))?;

    // Get substring before last dot
    let without_ext = &file_path[..last_dot];

    // Find last path separator
    let name_start = without_ext.rfind(['\\', '/', ':'])
        .map(|pos| pos + 1)
        .unwrap_or(0);

    let name = &without_ext[name_start..];

    if name.len() > 15 {
        bail!("Market name ({}) is too long", name);
    }

    Ok(name.to_string())
}

fn read_market_file(file_path: &str, market_name: &str) -> Result<MarketData> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open market file {}", file_path))?;
    let reader = BufReader::new(file);

    let mut market = MarketData::new(market_name.to_string());
    let mut prior_date = 0i32;

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split([',', ' ', '\t'])
            .filter(|s| !s.is_empty())
            .collect();

        if fields.is_empty() {
            continue;
        }

        // Parse date
        let full_date: i32 = fields[0].parse()
            .with_context(|| format!("Invalid date in {} line {}", file_path, line_num + 1))?;

        let year = full_date / 10000;
        let month = (full_date % 10000) / 100;
        let day = full_date % 100;

        if !(1..=12).contains(&month) || !(1..=31).contains(&day) || !(1800..=2030).contains(&year) {
            bail!("Invalid date {} in market file {} line {}", full_date, file_path, line_num + 1);
        }

        if full_date <= prior_date {
            bail!("Date failed to increase in market file {} line {}", file_path, line_num + 1);
        }

        prior_date = full_date;

        // Parse OHLC (we only need close, but validate all)
        let open: f64 = if fields.len() > 1 {
            fields[1].parse().unwrap_or(0.0)
        } else {
            0.0
        };

        let high: f64 = if fields.len() > 2 {
            fields[2].parse().unwrap_or(open)
        } else {
            open
        };

        let low: f64 = if fields.len() > 3 {
            fields[3].parse().unwrap_or(open)
        } else {
            open
        };

        let close: f64 = if fields.len() > 4 {
            fields[4].parse().unwrap_or(open)
        } else {
            open
        };

        if high < open || high < close || low > open || low > close {
            bail!("Open or close outside high/low bounds in market file {} line {}", 
                  file_path, line_num + 1);
        }

        market.dates.push(full_date);
        market.close.push(close);
    }

    if market.dates.is_empty() {
        bail!("No data read from market file {}", file_path);
    }

    Ok(market)
}

/// Align dates across all markets, keeping only dates present in all markets
pub fn align_dates(markets: &mut Vec<MarketData>) -> usize {
    if markets.is_empty() {
        return 0;
    }

    let n_markets = markets.len();
    let mut market_indices = vec![0usize; n_markets];
    let mut aligned_data: Vec<Vec<(i32, f64)>> = vec![Vec::new(); n_markets];

    loop {
        // Find max date at current index of each market
        let mut max_date = 0i32;
        for i in 0..n_markets {
            if market_indices[i] < markets[i].dates.len() {
                let date = markets[i].dates[market_indices[i]];
                if date > max_date {
                    max_date = date;
                }
            }
        }

        if max_date == 0 {
            break; // All markets exhausted
        }

        // Advance all markets until they reach or pass max date
        let mut all_same_date = true;
        for i in 0..n_markets {
            while market_indices[i] < markets[i].dates.len() {
                let date = markets[i].dates[market_indices[i]];
                if date >= max_date {
                    break;
                }
                market_indices[i] += 1;
            }

            if market_indices[i] >= markets[i].dates.len() {
                all_same_date = false;
                break;
            }

            let date = markets[i].dates[market_indices[i]];
            if date != max_date {
                all_same_date = false;
            }
        }

        // Check if any market ran out
        if market_indices.iter().zip(markets.iter())
            .any(|(idx, market)| *idx >= market.dates.len()) {
            break;
        }

        // If we have a complete set for this date, grab it
        if all_same_date {
            for i in 0..n_markets {
                let date = markets[i].dates[market_indices[i]];
                let close = markets[i].close[market_indices[i]];
                aligned_data[i].push((date, close));
                market_indices[i] += 1;
            }
        }
    }

    let n_cases = aligned_data[0].len();

    // Replace market data with aligned data
    for (i, market) in markets.iter_mut().enumerate() {
        market.dates.clear();
        market.close.clear();
        
        for (date, close) in &aligned_data[i] {
            market.dates.push(*date);
            market.close.push(*close);
        }
    }

    n_cases
}

/// Convert all closing prices to log prices
pub fn convert_to_log_prices(markets: &mut [MarketData]) {
    for market in markets.iter_mut() {
        for price in market.close.iter_mut() {
            *price = price.ln();
        }
    }
}
