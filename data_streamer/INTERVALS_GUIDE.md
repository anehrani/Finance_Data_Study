# Historical Data Downloader - Usage Guide

## Quick Start

### Download Daily Data (Default)
```bash
cargo run --bin download_historical
```

### Download Hourly Data
```bash
cargo run --bin download_historical -- --interval 60
```

### Download 15-Minute Data
```bash
cargo run --bin download_historical -- --interval 15
```

## Command-Line Options

```
Options:
  -i, --interval <INTERVAL>  Time interval [default: D]
  -l, --limit <LIMIT>        Number of bars to download (max 1000) [default: 1000]
      --spot                 Download spot assets [default: true]
      --linear               Download linear assets [default: true]
  -h, --help                 Print help
```

## Supported Intervals

| Interval | Description | Example Command |
|----------|-------------|-----------------|
| `1` | 1 minute | `--interval 1` |
| `3` | 3 minutes | `--interval 3` |
| `5` | 5 minutes | `--interval 5` |
| `15` | 15 minutes | `--interval 15` |
| `30` | 30 minutes | `--interval 30` |
| `60` | 1 hour | `--interval 60` |
| `120` | 2 hours | `--interval 120` |
| `240` | 4 hours | `--interval 240` |
| `360` | 6 hours | `--interval 360` |
| `720` | 12 hours | `--interval 720` |
| `D` | Daily | `--interval D` (default) |
| `W` | Weekly | `--interval W` |
| `M` | Monthly | `--interval M` |

### Alternative Formats
You can also use these formats:
- `1h` or `60m` for 1 hour
- `1d` or `daily` for daily
- `1w` or `weekly` for weekly
- `1M` or `monthly` for monthly

## Examples

### 1. Download Last 1000 Hours of Data
```bash
cargo run --bin download_historical -- --interval 60 --limit 1000
```

**Output**:
```
historical_data/
├── spot/
│   └── 1hour/
│       ├── MARKETS.TXT
│       ├── AAPLXUSDT.TXT    # 1000 hourly bars
│       └── ...
└── linear/
    └── 1hour/
        ├── MARKETS.TXT
        └── ...
```

### 2. Download Last 500 Days of Daily Data
```bash
cargo run --bin download_historical -- --interval D --limit 500
```

### 3. Download 15-Minute Bars for Intraday Analysis
```bash
cargo run --bin download_historical -- --interval 15
```

**Output**:
```
historical_data/
├── spot/
│   └── 15min/
│       ├── MARKETS.TXT
│       ├── AAPLXUSDT.TXT    # Up to 1000 15-min bars
│       └── ...
```

### 4. Download Only Spot Assets (Hourly)
```bash
cargo run --bin download_historical -- --interval 60 --linear false
```

### 5. Download Only Linear Assets (4-Hour)
```bash
cargo run --bin download_historical -- --interval 240 --spot false
```

### 6. Download Multiple Intervals
```bash
# Daily data
cargo run --bin download_historical -- --interval D

# Hourly data
cargo run --bin download_historical -- --interval 60

# 15-minute data
cargo run --bin download_historical -- --interval 15
```

This creates separate directories for each interval:
```
historical_data/
├── spot/
│   ├── daily/
│   ├── 1hour/
│   └── 15min/
└── linear/
    ├── daily/
    ├── 1hour/
    └── 15min/
```

## Data Format

### Daily/Weekly/Monthly Data
```
YYYYMMDD Open High Low Close
20250701 184.65 217.62 184.65 212.69
20250702 212.69 215.71 208.00 215.71
```

### Intraday Data (Minutes/Hours)
```
YYYYMMDD HH:MM:SS Open High Low Close
20251130 14:00:00 278.50 279.00 278.00 278.75
20251130 15:00:00 278.75 279.50 278.50 279.25
```

## Data Availability by Interval

### How Much Data Can You Get?

| Interval | Max Bars | Time Coverage | Example |
|----------|----------|---------------|---------|
| 1 min | 1000 | ~16.7 hours | Last 16 hours |
| 5 min | 1000 | ~3.5 days | Last 3 days |
| 15 min | 1000 | ~10.4 days | Last 10 days |
| 1 hour | 1000 | ~41.7 days | Last 41 days |
| 4 hour | 1000 | ~166.7 days | Last 166 days |
| Daily | 1000 | ~2.7 years | Last 1000 days |

**Note**: Actual availability depends on when the asset was listed. Most tokenized stocks only have data since July 2025 (~150 days).

## Use Cases

### Backtesting (Daily)
```bash
cargo run --bin download_historical -- --interval D
./backtest historical_data/spot/daily/MARKETS.TXT
```

### Intraday Strategy Development (Hourly)
```bash
cargo run --bin download_historical -- --interval 60
python analyze_hourly.py historical_data/spot/1hour/
```

### High-Frequency Analysis (15-min)
```bash
cargo run --bin download_historical -- --interval 15
```

### Multi-Timeframe Analysis
```bash
# Download all timeframes
cargo run --bin download_historical -- --interval D
cargo run --bin download_historical -- --interval 240
cargo run --bin download_historical -- --interval 60
cargo run --bin download_historical -- --interval 15
```

## Tips

1. **Start with Daily**: Get the full history first
   ```bash
   cargo run --bin download_historical -- --interval D
   ```

2. **Then Get Intraday**: Download shorter intervals for recent data
   ```bash
   cargo run --bin download_historical -- --interval 60
   ```

3. **Check Data Availability**: Some assets may not have data for all intervals
   - Newer assets: Limited history
   - Older assets: More complete data

4. **Combine with Live Streaming**: Use historical data for backtesting, live streaming for real-time
   ```bash
   # Historical for backtesting
   cargo run --bin download_historical -- --interval D
   
   # Live for current trading
   cargo run --bin stream_live
   ```

## Troubleshooting

### "No data available"
- Asset may not have data for that interval
- Try a longer interval (e.g., daily instead of hourly)

### "Error fetching data"
- Check internet connection
- Bybit API may be rate-limited (wait a moment and retry)

### Want more than 1000 bars?
- Bybit API limit is 1000 bars per request
- For more data, you'll need to make multiple requests with different time ranges
- Or use live streaming to accumulate data over time

---

**Summary**: You can now download TradFi data in any interval from 1-minute to monthly, with up to 1000 bars per download!
