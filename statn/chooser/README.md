# Chooser - Market Selection System

A Rust implementation of a nested walkforward trading system that selects the best-performing market based on multiple performance criteria.

## Overview

This package provides two main modes:

1. **chooser** - Monte Carlo permutation testing version
2. **chooser_dd** - Drawdown analysis version

Both modes implement a nested walkforward system that:
- Evaluates multiple markets using different performance criteria (total return, Sharpe ratio, profit factor)
- Selects the best criterion based on recent out-of-sample performance
- Uses that criterion to select the best market for the next period
- Tracks performance and generates detailed reports

## Usage

### Chooser Mode (Monte Carlo Permutation Testing)

```bash
cargo run -p chooser -- chooser <file_list> <is_n> <oos1_n> <nreps>
```

**Arguments:**
- `file_list`: Path to text file containing list of market history files
- `is_n`: Number of market history records for each selection criterion to analyze
- `oos1_n`: Number of OOS records for choosing best criterion
- `nreps`: Number of Monte-Carlo replications (1 or 0 for none)

**Example:**
```bash
cargo run -p chooser -- chooser markets.txt 1000 100 3
```

### Chooser DD Mode (Drawdown Analysis)

```bash
cargo run -p chooser -- chooser_dd <file_list> <is_n> <oos1_n>
```

**Arguments:**
- `file_list`: Path to text file containing list of market history files
- `is_n`: Number of market history records for each selection criterion to analyze
- `oos1_n`: Number of OOS records for choosing best criterion

**Example:**
```bash
cargo run -p chooser -- chooser_dd markets.txt 1000 100
```

## Input File Format

### Market List File
A text file with one market data file path per line:
```
/path/to/market1.csv
/path/to/market2.csv
/path/to/market3.csv
```

### Market Data Files
CSV format with columns: Date, Open, High, Low, Close
- Date format: YYYYMMDD (e.g., 20230115)
- Prices can be separated by commas, spaces, or tabs
- Dates must be in ascending order
- All markets will be date-aligned (only common dates kept)

Example:
```
20230101,100.5,101.2,99.8,100.9
20230102,100.9,102.1,100.5,101.5
20230103,101.5,102.0,101.0,101.8
```

## Output

Both modes write results to `CHOOSER.LOG` in the current directory, including:
- Market performance statistics
- Criterion selection frequencies
- Final system performance
- P-values (chooser mode) or drawdown bounds (chooser_dd mode)

## Performance Criteria

The system evaluates three performance criteria:

1. **Total Return**: Simple log return over the period
2. **Sharpe Ratio**: Risk-adjusted return measure
3. **Profit Factor**: Ratio of winning to losing trades

## Implementation Details

- All prices are converted to log prices for computational efficiency
- Markets are date-aligned to ensure synchronized data
- Permutation testing preserves correlations across markets
- Drawdown analysis uses bootstrap confidence intervals
