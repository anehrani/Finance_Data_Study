# Historical Stock Data Downloaders

This project includes two Python scripts for downloading historical stock data:

1. **`download_yahoo_data.py`** - **RECOMMENDED** - Uses Yahoo Finance (yfinance library)
2. **`download_investing_data.py`** - Uses Investing.com (investpy library)

## ⭐ Recommended: Yahoo Finance Downloader

The Yahoo Finance downloader is **more reliable** and doesn't face the same rate limiting issues as investing.com.

### Installation

Install the required dependencies:

```bash
.venv/bin/pip install yfinance pandas
```

## Usage

### Step 1: Create a stocks list file

Create a file named `stocks_list.txt` in the project root with the following format:

```
# Stock list format (Yahoo Finance symbols):
# One symbol per line. Lines starting with # are comments.

# US Stocks
AAPL
MSFT
GOOGL
AMZN
TSLA
NVDA
META
JPM
V
WMT

# International stocks (use Yahoo Finance symbols):
# 7203.T      # Toyota (Tokyo)
# SAP         # SAP (NYSE)
# HSBA.L      # HSBC (London)
# NESN.SW     # Nestle (Switzerland)
# 005930.KS   # Samsung (Korea)
```

**Format:**
- One stock per line
- Lines starting with `#` are comments and will be ignored
- Use Yahoo Finance ticker symbols (e.g., `.T` for Tokyo, `.L` for London)

### Step 2: Run the script

```bash
.venv/bin/python download_yahoo_data.py
```

The script will:
1. Read the stock symbols from `stocks_list.txt`
2. Download historical data from **1995-01-01** to **today**
3. Save each stock's data as a CSV file in the `data/historical_data/` directory
4. Display progress and summary statistics

## Output

Downloaded data will be saved in the `data/historical_data/` directory with filenames in the format:
```
SYMBOL.csv
```

For example:
- `AAPL.csv`
- `MSFT.csv`
- `7203.T.csv` (Toyota)

Each CSV file contains the following columns:
- **Date** (index)
- **Open**
- **High**
- **Low**
- **Close**
- **Volume**
- **Currency**

## Configuration

You can modify the following variables in `download_yahoo_data.py`:

```python
input_file = "stocks_list.txt"        # Input file with stock symbols
output_dir = "data/historical_data"   # Output directory
start_date = "1995-01-01"             # Start date (YYYY-MM-DD)
end_date = datetime.now().strftime("%Y-%m-%d")  # End date (defaults to today)
```

## Yahoo Finance Symbol Format

Yahoo Finance uses specific suffixes for international exchanges:

- **Tokyo Stock Exchange**: `.T` (e.g., `7203.T` for Toyota)
- **London Stock Exchange**: `.L` (e.g., `HSBA.L` for HSBC)
- **Swiss Exchange**: `.SW` (e.g., `NESN.SW` for Nestle)
- **Korea Exchange**: `.KS` (e.g., `005930.KS` for Samsung)
- **Hong Kong**: `.HK` (e.g., `0700.HK` for Tencent)
- **Frankfurt**: `.DE` (e.g., `SAP.DE` for SAP)

For a complete list, refer to the [Yahoo Finance documentation](https://help.yahoo.com/kb/SLN2310.html).

## Example Output

```
Reading stock list from 'stocks_list.txt'...
Found 3 stock(s) to download.
Date range: 1995-01-01 to 2025-12-02
Output directory: data/historical_data
------------------------------------------------------------
Downloading AAPL...
  ✓ Successfully downloaded 7781 rows to data/historical_data/AAPL.csv
    Date range: 1995-01-03 to 2025-12-01

Downloading MSFT...
  ✓ Successfully downloaded 7781 rows to data/historical_data/MSFT.csv
    Date range: 1995-01-03 to 2025-12-01

Downloading GOOGL...
  ✓ Successfully downloaded 5356 rows to data/historical_data/GOOGL.csv
    Date range: 2004-08-19 to 2025-12-01

------------------------------------------------------------
Download complete!
  Successful: 3
  Failed: 0
  Total: 3

Data saved to: data/historical_data/
```

## Troubleshooting

### Stock not found error
If you get an error like "No data available", verify:
1. The stock symbol is correct (use Yahoo Finance format)
2. The stock ticker is valid on Yahoo Finance
3. Check [Yahoo Finance](https://finance.yahoo.com) to confirm the correct symbol

### Date range issues
Some stocks may not have data going back to 1995. The script will download whatever data is available and show the actual date range.

---

## Alternative: Investing.com Downloader

If you prefer to use investing.com as a data source, you can use `download_investing_data.py`:

### Installation

```bash
.venv/bin/pip install investpy pandas
```

### Usage

The investing.com script uses a different format for the stocks list:

```
# Format: SYMBOL,country
AAPL,united states
TOYOTA,japan
SAP,germany
```

Run with:
```bash
.venv/bin/python download_investing_data.py
```

### ⚠️ Known Issues

The investing.com downloader may encounter:
- **HTTP 403 errors** due to anti-scraping measures
- **Rate limiting** when downloading many stocks
- **Inconsistent availability** of data

For these reasons, **Yahoo Finance is the recommended option**.

---

## Notes

- Yahoo Finance data is provided for research and educational purposes
- The `yfinance` library is an unofficial library and not affiliated with Yahoo
- Data quality and availability may vary by stock and exchange
- The script downloads daily timeframe data by default
- Always verify critical data with official sources

## License

This script is part of the Hilbert_project_01 and follows the project's license.
