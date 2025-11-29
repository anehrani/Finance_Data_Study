# Backtesting Implementation Summary

## Overview

Added a comprehensive backtesting system to `try_cd_comb` that simulates trading based on the model's predictions. The system generates detailed performance metrics and a trade log, writing them to a separate results file.

## What Was Added

### 1. Backtesting Module (`src/backtest.rs`)
- **`generate_signals`**: Converts model predictions into trading signals (1=Buy, -1=Sell, 0=Hold).
- **`run_backtest`**: Orchestrates the backtest using the shared `backtesting` library.
- **Integration**: Bridges `try_cd_comb`'s `CoordinateDescent` model with the `backtesting` crate.

### 2. Result Reporting (`src/evaluation.rs`)
- **`write_backtest_results`**: Formats and writes `TradeStats` to a text file.
- **Report Content**:
  - **Performance Summary**: Initial/Final Budget, Total P&L, ROI, Costs, Drawdown, Sharpe Ratio.
  - **Trade Statistics**: Total trades, Wins/Losses, Win Rate.
  - **Trade Log**: Detailed table of every trade (Entry/Exit prices, P&L, Return %).

### 3. Main Integration (`main.rs`)
- **Execution**: Runs backtest after model evaluation.
- **Data Handling**: Extracts correct test prices (log-space) for the backtester.
- **Output**: Writes results to `backtest_results.txt` (or similar, based on output filename).

### 4. Dependencies
- Added `backtesting` crate to `Cargo.toml`.

## Usage

The backtest runs automatically when you execute `try_cd_comb`.

```bash
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt --rsi-periods 14,21,28 --include-macd
```

## Output Example (`backtest_results.txt`)

```text
Backtest Results
================

Performance Summary
-------------------
Initial Budget:   $10000.00
Final Budget:     $15453.20
Total P&L:        $5453.20
Return (ROI):     54.53%
Total Costs:      $123.45
Max Drawdown:     12.50%
Sharpe Ratio:     1.8500

Trade Statistics
----------------
Total Trades:     45
Winning Trades:   28
Losing Trades:    17
Win Rate:         62.22%

Trade Log
---------
Type   Entry Idx  Entry Price Exit Idx   Exit Price P&L          Return %  
LONG   10         100.5000    15         102.3000   179.1000     1.79      
SHORT  15         102.3000    22         101.1000   118.5000     1.17      
...
```

## Technical Details

- **Initial Budget**: $10,000 (hardcoded for now).
- **Transaction Cost**: 0.1% per trade (hardcoded).
- **Signal Logic**:
  - Prediction > 0 → BUY (Long)
  - Prediction < 0 → SELL (Short)
  - Prediction = 0 → HOLD (Flat)
- **Price Handling**: The backtester handles log-to-linear price conversion internally.

## Files Modified/Created

- `src/backtest.rs` (Created)
- `src/evaluation.rs` (Modified)
- `src/lib.rs` (Modified)
- `main.rs` (Modified)
- `Cargo.toml` (Modified)
- `README.md` (Modified)
