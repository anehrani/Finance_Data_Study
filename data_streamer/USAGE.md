# Bybit TradFi Data Tools

Two separate tools for different use cases:

## 1. Historical Data Downloader

Downloads all available historical daily data for TradFi assets.

### Usage
```bash
cargo run --bin download_historical
```

### What it does
- Fetches all available daily OHLC data (up to 1000 days per symbol)
- Creates `historical_data/spot/` and `historical_data/linear/` directories
- Generates `MARKETS.TXT` files for backtesting
- One-time download, then exits

### Output
```
historical_data/
├── spot/
│   ├── MARKETS.TXT
│   ├── AAPLXUSDT.TXT      # ~150 days (since July 2025)
│   ├── TSLAXUSDT.TXT      # ~146 days
│   └── ...
└── linear/
    ├── MARKETS.TXT
    ├── XAUTUSDT.TXT        # ~234 days
    └── ...
```

### Data Format
```
YYYYMMDD Open High Low Close
20250701 184.65 217.62 184.65 212.69
20250702 212.69 215.71 208.00 215.71
```

## 2. Live Data Streamer

Streams real-time tick data continuously via WebSocket.

### Usage
```bash
cargo run --bin stream_live
```

### What it does
- Connects to Bybit WebSocket
- Captures every trade in real-time
- Aggregates ticks into 1-minute OHLCV bars
- Runs continuously until stopped (Ctrl+C)

### Output
```
tick_data/
├── spot/
│   ├── AAPLXUSDT.txt      # Every trade: timestamp,price,volume,side
│   └── ...
└── linear/
    ├── XAUTUSDT.txt
    └── ...

bar_data/
├── spot/
│   ├── AAPLXUSDT.txt      # 1-min bars: YYYYMMDD HH:MM:SS O H L C V
│   └── ...
└── linear/
    ├── XAUTUSDT.txt
    └── ...
```

### Tick Data Format
```
1732978103456,5950.25,0.5,Buy
1732978103789,5950.50,1.2,Sell
```

### Bar Data Format
```
20251130 23:45:00 5950.00 5951.00 5949.50 5950.25 15.3
20251130 23:46:00 5950.25 5952.00 5950.00 5951.50 22.1
```

## Historical Data Availability

### Why Only ~150-375 Days?

**These tokenized stocks are NEW!** They only started trading on Bybit in mid-2025:

| Symbol | Available Days | Start Date | Reason |
|--------|---------------|------------|---------|
| AAPLXUSDT | ~153 | July 2025 | New listing |
| TSLAXUSDT | ~146 | July 2025 | New listing |
| NVDAXUSDT | ~154 | July 2025 | New listing |
| GOOGLXUSDT | ~147 | July 2025 | New listing |
| METAXUSDT | ~152 | July 2025 | New listing |
| AMZNXUSDT | ~147 | July 2025 | New listing |
| SPXUSDT | ~375 | Earlier | Older listing |
| XAUTUSDT | ~234 | Earlier | Older listing |
| GASUSDT | ~437 | Earlier | Older listing |

**This is ALL the data that exists** - Bybit doesn't have historical data before these assets were listed.

### For More Historical Data

If you need more history:
1. **Use traditional data sources**: Yahoo Finance, Alpha Vantage, etc. for the underlying stocks
2. **Wait**: As time passes, more data will accumulate
3. **MT5**: Bybit's MT5 platform may have different data availability

## Quick Start

### 1. Download Historical Data (One-time)
```bash
cargo run --bin download_historical
```

Wait for completion (~30 seconds), then use the data for backtesting.

### 2. Stream Live Data (Continuous)
```bash
cargo run --bin stream_live
```

Let it run for as long as you want to collect data. Press Ctrl+C to stop.

## Use Cases

### Backtesting
```bash
# Download historical data first
cargo run --bin download_historical

# Use with your backtesting system
./backtest historical_data/spot/MARKETS.TXT
```

### Live Trading/Analysis
```bash
# Stream live data
cargo run --bin stream_live

# In another terminal, analyze the data
tail -f tick_data/spot/AAPLXUSDT.txt
```

### Research
```bash
# Collect data for a day
cargo run --bin stream_live
# (Let it run for 24 hours, then Ctrl+C)

# Analyze the collected ticks
python analyze_ticks.py tick_data/spot/AAPLXUSDT.txt
```

## Summary

- **Historical Downloader**: One-time download of all available daily data
- **Live Streamer**: Continuous real-time tick collection
- **Data Availability**: Limited by when assets were listed (~150-437 days)
- **All TradFi Assets**: 11 spot + 3 linear = 14 total
