use data_streamer::bybit::BybitClient;
use chrono::{DateTime, Utc, Duration};
use clap::Parser;
use reqwest::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::collections::HashSet;

#[derive(Parser, Debug)]
#[command(name = "download_historical")]
#[command(about = "Download historical TradFi data from Bybit", long_about = None)]
struct Args {
    /// Time interval: 1, 3, 5, 15, 30, 60, 120, 240, 360, 720, D, W, M
    #[arg(short, long, default_value = "D")]
    interval: String,

    /// Total number of data points to download (will make multiple API calls if > 1000)
    #[arg(short, long, default_value = "1000")]
    limit: usize,

    /// Download spot assets
    #[arg(long, default_value = "true")]
    spot: bool,

    /// Download linear assets
    #[arg(long, default_value = "true")]
    linear: bool,
}

fn interval_to_string(interval: &str) -> &str {
    match interval {
        "1" | "1m" => "1",
        "3" | "3m" => "3",
        "5" | "5m" => "5",
        "15" | "15m" => "15",
        "30" | "30m" => "30",
        "60" | "1h" | "60m" => "60",
        "120" | "2h" => "120",
        "240" | "4h" => "240",
        "360" | "6h" => "360",
        "720" | "12h" => "720",
        "D" | "1d" | "daily" => "D",
        "W" | "1w" | "weekly" => "W",
        "M" | "1M" | "monthly" => "M",
        _ => "D",
    }
}

fn interval_to_dirname(interval: &str) -> String {
    match interval {
        "1" => "1min".to_string(),
        "3" => "3min".to_string(),
        "5" => "5min".to_string(),
        "15" => "15min".to_string(),
        "30" => "30min".to_string(),
        "60" => "1hour".to_string(),
        "120" => "2hour".to_string(),
        "240" => "4hour".to_string(),
        "360" => "6hour".to_string(),
        "720" => "12hour".to_string(),
        "D" => "daily".to_string(),
        "W" => "weekly".to_string(),
        "M" => "monthly".to_string(),
        _ => "daily".to_string(),
    }
}

fn interval_to_millis(interval: &str) -> i64 {
    match interval {
        "1" => 60_000,           // 1 minute
        "3" => 180_000,          // 3 minutes
        "5" => 300_000,          // 5 minutes
        "15" => 900_000,         // 15 minutes
        "30" => 1_800_000,       // 30 minutes
        "60" => 3_600_000,       // 1 hour
        "120" => 7_200_000,      // 2 hours
        "240" => 14_400_000,     // 4 hours
        "360" => 21_600_000,     // 6 hours
        "720" => 43_200_000,     // 12 hours
        "D" => 86_400_000,       // 1 day
        "W" => 604_800_000,      // 1 week
        "M" => 2_592_000_000,    // ~30 days
        _ => 86_400_000,
    }
}

fn format_timestamp(interval: &str, ts_millis: i64) -> String {
    if let Some(dt) = DateTime::<Utc>::from_timestamp_millis(ts_millis) {
        match interval {
            "D" | "W" | "M" => dt.format("%Y%m%d").to_string(),
            _ => dt.format("%Y%m%d %H:%M:%S").to_string(),
        }
    } else {
        String::new()
    }
}

async fn download_kline_batch(
    symbol: &str,
    interval: &str,
    limit: usize,
    end_time: Option<i64>,
) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let url = "https://api.bybit.com/v5/market/kline";
    let limit_str = limit.to_string();
    let end_time_str = end_time.map(|et| et.to_string());
    
    let mut query_params: Vec<(&str, &str)> = vec![
        ("category", "spot"),
        ("symbol", symbol),
        ("interval", interval),
        ("limit", &limit_str),
    ];
    
    if let Some(ref et_str) = end_time_str {
        query_params.push(("end", et_str));
    }
    
    let response = reqwest::Client::new()
        .get(url)
        .query(&query_params)
        .send()
        .await?;

    if response.status().is_success() {
        let api_response: serde_json::Value = response.json().await?;
        
        if api_response["retCode"].as_i64() == Some(0) {
            let list = api_response["result"]["list"].as_array()
                .ok_or("No list in response")?;
            
            let klines: Vec<Vec<String>> = list.iter()
                .filter_map(|item| {
                    item.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                })
                .collect();
            
            Ok(klines)
        } else {
            Err(format!("API error: {}", api_response["retMsg"].as_str().unwrap_or("Unknown")).into())
        }
    } else {
        Err(format!("HTTP error: {}", response.status()).into())
    }
}

async fn download_kline_data_paginated(
    symbol: &str,
    interval: &str,
    total_limit: usize,
) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let mut all_klines = Vec::new();
    let mut seen_timestamps = HashSet::new();
    let batch_size = 1000; // Bybit max per request
    let mut end_time: Option<i64> = None;
    let interval_millis = interval_to_millis(interval);
    
    let num_batches = (total_limit + batch_size - 1) / batch_size;
    
    for batch_num in 0..num_batches {
        let remaining = total_limit - all_klines.len();
        if remaining == 0 {
            break;
        }
        
        let current_limit = remaining.min(batch_size);
        
        if batch_num > 0 {
            print!(".");
            std::io::stdout().flush().ok();
        }
        
        match download_kline_batch(symbol, interval, current_limit, end_time).await {
            Ok(klines) => {
                if klines.is_empty() {
                    break; // No more data available
                }
                
                // Filter out duplicates and add to collection
                for kline in klines {
                    if kline.len() > 0 {
                        if let Ok(ts) = kline[0].parse::<i64>() {
                            if seen_timestamps.insert(ts) {
                                all_klines.push(kline);
                            }
                        }
                    }
                }
                
                // Get the oldest timestamp for next batch
                if let Some(oldest) = all_klines.last() {
                    if let Ok(ts) = oldest[0].parse::<i64>() {
                        end_time = Some(ts - interval_millis);
                    }
                }
                
                // Small delay to avoid rate limiting
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    
    if num_batches > 1 {
        println!(); // New line after progress dots
    }
    
    Ok(all_klines)
}

async fn download_historical_data(
    symbols: &[String],
    category: &str,
    interval: &str,
    total_limit: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let interval_dir = interval_to_dirname(interval);
    let num_batches = (total_limit + 999) / 1000;
    
    println!("\n=== Downloading {} {} data for {} symbols ===", interval_dir, category, symbols.len());
    if total_limit > 1000 {
        println!("Will make {} API requests per symbol to get {} bars", num_batches, total_limit);
    }
    
    let hist_dir = Path::new("historical_data").join(category).join(&interval_dir);
    fs::create_dir_all(&hist_dir)?;
    
    let markets_path = hist_dir.join("MARKETS.TXT");
    let mut markets_file = File::create(&markets_path)?;
    
    for (idx, symbol) in symbols.iter().enumerate() {
        print!("[{}/{}] Downloading {} data for {}...", idx + 1, symbols.len(), interval_dir, symbol);
        std::io::stdout().flush().ok();
        
        match download_kline_data_paginated(symbol, interval, total_limit).await {
            Ok(klines) => {
                if klines.is_empty() {
                    println!(" No data available");
                    continue;
                }
                
                let file_path = hist_dir.join(format!("{}.TXT", symbol));
                let mut file = File::create(&file_path)?;
                
                let mut klines_rev = klines.clone();
                klines_rev.reverse();
                
                for kline in klines_rev {
                    if kline.len() < 5 {
                        continue;
                    }
                    
                    let timestamp_str = &kline[0];
                    let open = &kline[1];
                    let high = &kline[2];
                    let low = &kline[3];
                    let close = &kline[4];
                    
                    if let Ok(ts_millis) = timestamp_str.parse::<i64>() {
                        let date_str = format_timestamp(interval, ts_millis);
                        if !date_str.is_empty() {
                            writeln!(file, "{} {} {} {} {}", date_str, open, high, low, close)?;
                        }
                    }
                }
                
                if let Ok(abs_path) = fs::canonicalize(&file_path) {
                    writeln!(markets_file, "{}", abs_path.display())?;
                } else {
                    writeln!(markets_file, "{}", file_path.display())?;
                }
                
                println!(" ✓ {} bars", klines.len());
            }
            Err(e) => {
                println!(" ✗ Error: {}", e);
            }
        }
    }
    
    println!("\nData saved to: {}", hist_dir.display());
    println!("Markets file: {}", markets_path.display());
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let client = BybitClient::new();

    let interval = interval_to_string(&args.interval);
    let total_limit = args.limit;

    println!("=== Bybit TradFi Historical Data Downloader ===");
    println!("Interval: {} | Total bars: {}", interval_to_dirname(interval), total_limit);
    if total_limit > 1000 {
        println!("Note: Will make multiple API requests to fetch {} bars", total_limit);
    }
    println!();
    
    // Get Spot symbols
    let spot_symbols = if args.spot {
        println!("Fetching spot tickers...");
        match client.get_tickers("spot").await {
            Ok(tickers) => {
                let xstocks: Vec<String> = tickers
                    .iter()
                    .filter(|t| data_streamer::tradfi_filter::is_tradfi_symbol(&t.symbol))
                    .map(|t| t.symbol.clone())
                    .collect();
                println!("Found {} tokenized stocks", xstocks.len());
                xstocks
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // Get Linear symbols
    let linear_symbols = if args.linear {
        println!("Fetching linear tickers...");
        match client.get_tickers("linear").await {
            Ok(tickers) => {
                let tradfi: Vec<String> = tickers
                    .iter()
                    .filter(|t| {
                        let s = &t.symbol;
                        (s.contains("XAU") || s.contains("XAG") ||
                         s.contains("GAS") || s.contains("OIL") ||
                         (s.contains("SPX") && !s.contains("SPXL"))) &&
                        !s.contains("BANANA") && !s.contains("PERP")
                    })
                    .map(|t| t.symbol.clone())
                    .collect();
                println!("Found {} TradFi linear tickers", tradfi.len());
                tradfi
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // Download data
    if !spot_symbols.is_empty() {
        download_historical_data(&spot_symbols, "spot", interval, total_limit)
            .await
            .unwrap_or_else(|e| eprintln!("Error: {}", e));
    }
    
    if !linear_symbols.is_empty() {
        download_historical_data(&linear_symbols, "linear", interval, total_limit)
            .await
            .unwrap_or_else(|e| eprintln!("Error: {}", e));
    }

    println!("\n=== Download Complete ===");
    println!("\nExamples:");
    println!("  1000 bars:  cargo run --bin download_historical -- --interval 60 --limit 1000");
    println!("  2000 bars:  cargo run --bin download_historical -- --interval 60 --limit 2000");
    println!("  5000 bars:  cargo run --bin download_historical -- --interval 15 --limit 5000");

    Ok(())
}
