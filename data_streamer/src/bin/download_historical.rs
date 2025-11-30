use data_streamer::bybit::BybitClient;
use chrono::{DateTime, Utc};
use clap::Parser;
use reqwest::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "download_historical")]
#[command(about = "Download historical TradFi data from Bybit", long_about = None)]
struct Args {
    /// Time interval: 1, 3, 5, 15, 30, 60, 120, 240, 360, 720, D, W, M
    #[arg(short, long, default_value = "D")]
    interval: String,

    /// Number of data points to download (max 1000)
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
        _ => "D", // Default to daily
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

async fn download_kline_data(
    client: &BybitClient,
    symbol: &str,
    interval: &str,
    limit: usize,
) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    // Use the existing get_daily_kline but we'll need to modify it
    // For now, let's call it directly with the interval parameter
    // Note: The bybit module needs to be updated to support intervals
    
    let url = format!("https://api.bybit.com/v5/market/kline");
    let response = reqwest::Client::new()
        .get(&url)
        .query(&[
            ("category", "spot"),
            ("symbol", symbol),
            ("interval", interval),
            ("limit", &limit.to_string()),
        ])
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

async fn download_historical_data(
    client: &BybitClient,
    symbols: &[String],
    category: &str,
    interval: &str,
    limit: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let interval_dir = interval_to_dirname(interval);
    println!("\n=== Downloading {} {} data for {} ===", interval_dir, category, symbols.len());
    
    let hist_dir = Path::new("historical_data").join(category).join(&interval_dir);
    fs::create_dir_all(&hist_dir)?;
    
    // Create MARKETS.TXT
    let markets_path = hist_dir.join("MARKETS.TXT");
    let mut markets_file = File::create(&markets_path)?;
    
    for symbol in symbols {
        println!("Downloading {} data for {}...", interval_dir, symbol);
        
        match download_kline_data(client, symbol, interval, limit).await {
            Ok(klines) => {
                if klines.is_empty() {
                    println!("  No data available for {}", symbol);
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
                
                println!("  ✓ Downloaded {} bars for {}", klines.len(), symbol);
            }
            Err(e) => {
                eprintln!("  ✗ Error fetching data for {}: {}", symbol, e);
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    println!("Data saved to: {}", hist_dir.display());
    println!("Markets file: {}", markets_path.display());
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let client = BybitClient::new();

    let interval = interval_to_string(&args.interval);
    let limit = args.limit.min(1000); // Bybit max is 1000

    println!("=== Bybit TradFi Historical Data Downloader ===");
    println!("Interval: {} | Limit: {} bars\n", interval_to_dirname(interval), limit);
    
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
                println!("Found {} tokenized stocks\n", xstocks.len());
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
                println!("Found {} TradFi linear tickers\n", tradfi.len());
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
        download_historical_data(&client, &spot_symbols, "spot", interval, limit)
            .await
            .unwrap_or_else(|e| eprintln!("Error: {}", e));
    }
    
    if !linear_symbols.is_empty() {
        download_historical_data(&client, &linear_symbols, "linear", interval, limit)
            .await
            .unwrap_or_else(|e| eprintln!("Error: {}", e));
    }

    println!("\n=== Download Complete ===");
    println!("\nUsage examples:");
    println!("  Daily:   cargo run --bin download_historical -- --interval D");
    println!("  Hourly:  cargo run --bin download_historical -- --interval 60");
    println!("  15-min:  cargo run --bin download_historical -- --interval 15");
    println!("  Custom:  cargo run --bin download_historical -- --interval 60 --limit 500");

    Ok(())
}
