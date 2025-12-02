# Historical Data Download - Quick Start

## ✅ What Was Created

Two Python scripts for downloading historical stock data from 1995 to present:

1. **`download_yahoo_data.py`** ⭐ **RECOMMENDED**
   - Uses Yahoo Finance (yfinance library)
   - More reliable, no rate limiting issues
   - Successfully tested with AAPL, MSFT, GOOGL

2. **`download_investing_data.py`**
   - Uses Investing.com (investpy library)
   - May encounter HTTP 403 errors due to anti-scraping measures
   - Alternative option if Yahoo Finance doesn't have your data

## 🚀 Quick Start (Yahoo Finance - Recommended)

### 1. Install dependencies:
```bash
.venv/bin/pip install yfinance pandas
```

### 2. Create `stocks_list.txt`:
```
AAPL
MSFT
GOOGL
TSLA
```

### 3. Run the script:
```bash
.venv/bin/python download_yahoo_data.py
```

### 4. Find your data:
Data will be saved in `data/historical_data/` as CSV files:
- `AAPL.csv`
- `MSFT.csv`
- etc.

## 📊 Data Format

Each CSV contains:
- **Date** (index)
- **Open**
- **High**
- **Low**
- **Close**
- **Volume**
- **Dividends**
- **Stock Splits**

## 🌍 International Stocks

Use Yahoo Finance ticker symbols with exchange suffixes:

```
7203.T      # Toyota (Tokyo)
HSBA.L      # HSBC (London)
NESN.SW     # Nestle (Switzerland)
005930.KS   # Samsung (Korea)
0700.HK     # Tencent (Hong Kong)
SAP.DE      # SAP (Frankfurt)
```

## 📝 Test Results

Successfully downloaded:
- **AAPL**: 7,781 rows (1995-01-03 to 2025-12-01)
- **MSFT**: 7,781 rows (1995-01-03 to 2025-12-01)
- **GOOGL**: 5,356 rows (2004-08-19 to 2025-12-01)

## 📖 Full Documentation

See `INVESTING_DOWNLOADER_README.md` for complete documentation including:
- Configuration options
- Troubleshooting
- Alternative investing.com downloader
- Symbol format guide

## ⚠️ Notes

- Data is for research and educational purposes
- Some stocks may not have data back to 1995
- Always verify critical data with official sources
