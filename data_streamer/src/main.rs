mod bybit;
mod tradfi_filter;

use bybit::BybitClient;
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use reqwest::Error;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Debug, Deserialize)]
struct TradeData {
    #[serde(rename = "T")]
    timestamp: i64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "v")]
    volume: String,
    #[serde(rename = "S")]
    side: String,
}

#[derive(Debug, Deserialize)]
struct WsMessage {
    topic: String,
    #[serde(rename = "type")]
    msg_type: String,
    data: Vec<TradeData>,
}

#[derive(Clone)]
struct OHLCVBar {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

async fn subscribe_to_trades(
    url: &str,
    symbols: Vec<String>,
    category: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to {} WebSocket...", category);
    let (ws_stream, _) = connect_async(url).await?;
    println!("Connected to {}!", category);

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to trade streams for all symbols
    let mut topics = Vec::new();
    for symbol in &symbols {
        topics.push(format!("publicTrade.{}", symbol));
    }

    let subscribe_msg = json!({
        "op": "subscribe",
        "args": topics
    });

    write.send(Message::Text(subscribe_msg.to_string())).await?;
    println!("Subscribed to {} {} symbols", symbols.len(), category);

    // Create data directories
    let tick_dir = Path::new("tick_data").join(category);
    let bar_dir = Path::new("bar_data").join(category);
    fs::create_dir_all(&tick_dir)?;
    fs::create_dir_all(&bar_dir)?;

    // Create file handles for tick data
    let tick_files: Arc<Mutex<HashMap<String, File>>> = Arc::new(Mutex::new(HashMap::new()));
    let bar_files: Arc<Mutex<HashMap<String, File>>> = Arc::new(Mutex::new(HashMap::new()));
    
    // Track OHLCV bars (1-minute bars)
    let bars: Arc<Mutex<HashMap<String, OHLCVBar>>> = Arc::new(Mutex::new(HashMap::new()));

    for symbol in &symbols {
        let tick_path = tick_dir.join(format!("{}.txt", symbol));
        let bar_path = bar_dir.join(format!("{}.txt", symbol));
        
        let tick_file = File::create(&tick_path)?;
        let bar_file = File::create(&bar_path)?;
        
        tick_files.lock().await.insert(symbol.clone(), tick_file);
        bar_files.lock().await.insert(symbol.clone(), bar_file);
        
        println!("Created files for {}", symbol);
    }

    // Process incoming messages
    let mut tick_count = 0;
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    if ws_msg.msg_type == "snapshot" || ws_msg.msg_type == "delta" {
                        for trade in ws_msg.data {
                            let price: f64 = trade.price.parse().unwrap_or(0.0);
                            let volume: f64 = trade.volume.parse().unwrap_or(0.0);
                            
                            // Write tick data
                            let mut tick_files_lock = tick_files.lock().await;
                            if let Some(file) = tick_files_lock.get_mut(&trade.symbol) {
                                writeln!(
                                    file,
                                    "{},{},{},{}",
                                    trade.timestamp, trade.price, trade.volume, trade.side
                                )?;
                                tick_count += 1;
                                
                                if tick_count % 100 == 0 {
                                    println!("[{}] Received {} ticks", category, tick_count);
                                }
                            }
                            
                            // Update OHLCV bar
                            let minute_timestamp = (trade.timestamp / 60000) * 60000;
                            let mut bars_lock = bars.lock().await;
                            
                            let bar = bars_lock.entry(trade.symbol.clone()).or_insert(OHLCVBar {
                                timestamp: minute_timestamp,
                                open: price,
                                high: price,
                                low: price,
                                close: price,
                                volume: 0.0,
                            });
                            
                            // Check if we need to write the previous bar and start a new one
                            if bar.timestamp != minute_timestamp {
                                // Write completed bar
                                let mut bar_files_lock = bar_files.lock().await;
                                if let Some(file) = bar_files_lock.get_mut(&trade.symbol) {
                                    let dt = DateTime::<Utc>::from_timestamp_millis(bar.timestamp)
                                        .unwrap();
                                    writeln!(
                                        file,
                                        "{} {:.8} {:.8} {:.8} {:.8} {:.8}",
                                        dt.format("%Y%m%d %H:%M:%S"),
                                        bar.open,
                                        bar.high,
                                        bar.low,
                                        bar.close,
                                        bar.volume
                                    )?;
                                }
                                
                                // Start new bar
                                *bar = OHLCVBar {
                                    timestamp: minute_timestamp,
                                    open: price,
                                    high: price,
                                    low: price,
                                    close: price,
                                    volume: volume,
                                };
                            } else {
                                // Update current bar
                                bar.high = bar.high.max(price);
                                bar.low = bar.low.min(price);
                                bar.close = price;
                                bar.volume += volume;
                            }
                        }
                    }
                } else if text.contains("\"success\":true") {
                    println!("[{}] Subscription confirmed", category);
                } else if text.contains("ping") {
                    write.send(Message::Text(r#"{"op":"pong"}"#.to_string())).await?;
                }
            }
            Ok(Message::Ping(_)) => {
                write.send(Message::Pong(vec![])).await?;
            }
            Ok(Message::Close(_)) => {
                println!("[{}] WebSocket closed", category);
                break;
            }
            Err(e) => {
                eprintln!("[{}] Error receiving message: {}", category, e);
                break;
            }
            _ => {}
        }
    }

    println!("[{}] Total ticks received: {}", category, tick_count);
    Ok(())
}

async fn download_historical_data(
    client: &BybitClient,
    symbols: &[String],
    category: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Downloading historical data for {} ===", category);
    
    let hist_dir = Path::new("historical_data").join(category);
    fs::create_dir_all(&hist_dir)?;
    
    // Create MARKETS.TXT
    let markets_path = hist_dir.join("MARKETS.TXT");
    let mut markets_file = File::create(&markets_path)?;
    
    for symbol in symbols {
        println!("Downloading historical data for {}...", symbol);
        
        match client.get_daily_kline(symbol, 1000).await {
            Ok(klines) => {
                if klines.is_empty() {
                    println!("  No historical data available for {}", symbol);
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
                        if let Some(dt) = DateTime::<Utc>::from_timestamp_millis(ts_millis) {
                            let date_str = dt.format("%Y%m%d").to_string();
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
    
    println!("Historical data saved to: {}", hist_dir.display());
    println!("Markets file: {}", markets_path.display());
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = BybitClient::new();

    println!("=== Bybit TradFi Data Streamer ===\n");
    println!("=== Step 1: Identify TradFi assets ===");
    
    // Get Spot XUSDT tickers (tokenized stocks only, excluding crypto)
    println!("\nFetching spot tickers...");
    let spot_symbols = match client.get_tickers("spot").await {
        Ok(tickers) => {
            let xstocks: Vec<String> = tickers
                .iter()
                .filter(|t| tradfi_filter::is_tradfi_symbol(&t.symbol))
                .map(|t| t.symbol.clone())
                .collect();
            println!("Found {} tokenized stock tickers (TradFi only)", xstocks.len());
            for s in &xstocks {
                println!("  - {}", s);
            }
            xstocks
        }
        Err(e) => {
            eprintln!("Error fetching spot tickers: {}", e);
            Vec::new()
        }
    };

    // Get Linear tickers (indices, commodities, metals - excluding crypto)
    println!("\nFetching linear tickers...");
    let linear_symbols = match client.get_tickers("linear").await {
        Ok(tickers) => {
            let tradfi: Vec<String> = tickers
                .iter()
                .filter(|t| {
                    let s = &t.symbol;
                    // Include known TradFi patterns, exclude obvious crypto
                    (s.contains("XAU") || s.contains("XAG") || // Metals
                     s.contains("GAS") || s.contains("OIL") || // Energy
                     (s.contains("SPX") && !s.contains("SPXL")) || // Indices (exclude leveraged tokens)
                     s.contains("NAS100") || s.contains("DJI")) &&
                    !s.contains("BANANA") && // Exclude meme tokens
                    !s.contains("PERP") // Exclude perpetuals for now (or keep based on preference)
                })
                .map(|t| t.symbol.clone())
                .collect();
            println!("Found {} TradFi linear tickers (indices/commodities/metals)", tradfi.len());
            for s in &tradfi {
                println!("  - {}", s);
            }
            tradfi
        }
        Err(e) => {
            eprintln!("Error fetching linear tickers: {}", e);
            Vec::new()
        }
    };

    // Step 2: Download historical data
    println!("\n=== Step 2: Download historical data ===");
    
    if !spot_symbols.is_empty() {
        download_historical_data(&client, &spot_symbols, "spot")
            .await
            .unwrap_or_else(|e| eprintln!("Error downloading spot historical data: {}", e));
    }
    
    if !linear_symbols.is_empty() {
        download_historical_data(&client, &linear_symbols, "linear")
            .await
            .unwrap_or_else(|e| eprintln!("Error downloading linear historical data: {}", e));
    }

    // Step 3: Start real-time streaming
    println!("\n=== Step 3: Start real-time tick streaming ===");
    println!("Press Ctrl+C to stop\n");

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    // Start spot WebSocket
    if !spot_symbols.is_empty() {
        let spot_syms = spot_symbols.clone();
        let handle = tokio::spawn(async move {
            let url = "wss://stream.bybit.com/v5/public/spot";
            if let Err(e) = subscribe_to_trades(url, spot_syms, "spot").await {
                eprintln!("Spot WebSocket error: {}", e);
            }
        });
        handles.push(handle);
    }

    // Start linear WebSocket
    if !linear_symbols.is_empty() {
        let linear_syms = linear_symbols.clone();
        let handle = tokio::spawn(async move {
            let url = "wss://stream.bybit.com/v5/public/linear";
            if let Err(e) = subscribe_to_trades(url, linear_syms, "linear").await {
                eprintln!("Linear WebSocket error: {}", e);
            }
        });
        handles.push(handle);
    }

    // Wait for all WebSocket tasks
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}
