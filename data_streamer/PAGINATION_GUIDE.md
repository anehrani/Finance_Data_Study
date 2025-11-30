# Downloading More Than 1000 Bars

## The 1000-Bar Limit

Bybit's API has a **hard limit of 1000 bars per request**. However, the downloader now supports **automatic pagination** to download more data by making multiple requests.

## How It Works

When you request more than 1000 bars, the downloader:
1. Makes the first request for 1000 bars
2. Gets the timestamp of the oldest bar
3. Makes another request for bars before that timestamp
4. Repeats until it has all requested bars or runs out of data
5. Removes any duplicates

## Examples

### Download 2000 Hourly Bars (~83 days)
```bash
cargo run --bin download_historical -- --interval 60 --limit 2000
```

**Output**:
```
=== Bybit TradFi Historical Data Downloader ===
Interval: 1hour | Total bars: 2000
Note: Will make multiple API requests to fetch 2000 bars

[1/11] Downloading 1hour data for AAPLXUSDT... ✓ 2000 bars
[2/11] Downloading 1hour data for TSLAXUSDT... ✓ 2000 bars
...
```

### Download 5000 15-Minute Bars (~52 days)
```bash
cargo run --bin download_historical -- --interval 15 --limit 5000
```

### Download 3000 Daily Bars (~8.2 years)
```bash
cargo run --bin download_historical -- --interval D --limit 3000
```

**Note**: Most tokenized stocks only have ~150 days of data, so you'll get all available data (not 3000).

## How Many Requests?

| Total Bars | API Requests | Example |
|------------|--------------|---------|
| 1000 | 1 | Default |
| 2000 | 2 | `--limit 2000` |
| 3000 | 3 | `--limit 3000` |
| 5000 | 5 | `--limit 5000` |
| 10000 | 10 | `--limit 10000` |

Each request takes ~200ms, so:
- 2000 bars: ~2-3 seconds per symbol
- 5000 bars: ~5-6 seconds per symbol
- 10000 bars: ~10-12 seconds per symbol

## Maximum Data Available

### By Interval

| Interval | Max Bars | Time Coverage | Actual Availability* |
|----------|----------|---------------|---------------------|
| 1 min | Unlimited** | Any | Limited by listing date |
| 5 min | Unlimited** | Any | Limited by listing date |
| 15 min | Unlimited** | Any | Limited by listing date |
| 1 hour | Unlimited** | Any | ~3600 bars (~150 days) |
| 4 hour | Unlimited** | Any | ~900 bars (~150 days) |
| Daily | Unlimited** | Any | ~150-437 bars |

\* Most tokenized stocks listed in July 2025 (~150 days ago)
\** Limited only by when the asset was listed

### By Asset

| Asset | Listed | Max Daily Bars | Max Hourly Bars |
|-------|--------|----------------|-----------------|
| AAPLXUSDT | Jul 2025 | ~153 | ~3672 |
| TSLAXUSDT | Jul 2025 | ~146 | ~3504 |
| SPXUSDT | Earlier | ~375 | ~9000 |
| XAUTUSDT | Earlier | ~234 | ~5616 |
| GASUSDT | Earlier | ~437 | ~10488 |

## Practical Examples

### Get All Available Hourly Data
```bash
# Request more than exists to get everything
cargo run --bin download_historical -- --interval 60 --limit 10000
```

Result: You'll get all available data (e.g., ~3600 bars for AAPL, ~10000 for GAS)

### Get Last 3 Months of 15-Min Data
```bash
# 3 months = ~90 days = 8640 15-min bars
cargo run --bin download_historical -- --interval 15 --limit 8640
```

### Get Last 6 Months of Hourly Data
```bash
# 6 months = ~180 days = 4320 hourly bars
cargo run --bin download_historical -- --interval 60 --limit 4320
```

### Get Last Year of 4-Hour Data
```bash
# 1 year = ~365 days = 2190 4-hour bars
cargo run --bin download_historical -- --interval 240 --limit 2190
```

## Progress Indication

When downloading > 1000 bars, you'll see progress dots:
```
[1/11] Downloading 1hour data for AAPLXUSDT..... ✓ 5000 bars
```

Each dot represents one API request (1000 bars).

## Rate Limiting

The downloader includes:
- 200ms delay between requests to the same symbol
- 100ms delay between different symbols
- Automatic deduplication of overlapping data

This ensures you won't hit Bybit's rate limits even when downloading large amounts of data.

## Tips

1. **Start Small**: Test with 2000 bars first
   ```bash
   cargo run --bin download_historical -- --interval 60 --limit 2000
   ```

2. **Get Everything**: Request more than exists
   ```bash
   cargo run --bin download_historical -- --interval 60 --limit 20000
   ```

3. **Different Intervals**: Download multiple timeframes
   ```bash
   # Daily (all history)
   cargo run --bin download_historical -- --interval D --limit 10000
   
   # Hourly (last ~6 months)
   cargo run --bin download_historical -- --interval 60 --limit 5000
   
   # 15-min (last ~1 month)
   cargo run --bin download_historical -- --interval 15 --limit 3000
   ```

4. **Monitor Progress**: The tool shows progress for each symbol
   ```
   [3/11] Downloading 1hour data for NVDAXUSDT..... ✓ 5000 bars
   ```

## Summary

✅ **Yes, you can get more than 1000 bars!**
✅ **Automatic pagination** - just specify `--limit`
✅ **No manual work** - the tool handles everything
✅ **Rate-limit safe** - built-in delays
✅ **Deduplication** - no duplicate data

**Example**: Get 5000 hourly bars (~208 days)
```bash
cargo run --bin download_historical -- --interval 60 --limit 5000
```
