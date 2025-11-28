# try_diff_ev

A comprehensive trading system that uses differential evolution to optimize moving average crossover strategies for financial markets.

## Overview

This program implements a sophisticated trading system that combines technical analysis with evolutionary optimization. It uses moving average crossover signals to generate BUY/SELL/HOLD signals and employs differential evolution to find optimal parameters for maximum profitability.

The system supports multiple signal generation algorithms and includes comprehensive backtesting with transaction costs, risk metrics, and performance visualization.

## Features

- **Multiple Signal Generators**: Original and enhanced moving average crossover algorithms
- **Differential Evolution Optimization**: Evolutionary algorithm to find optimal trading parameters
- **Comprehensive Backtesting**: Realistic simulation with transaction costs and position tracking
- **Risk Analysis**: Drawdown analysis, Sharpe ratio, and win rate calculations
- **Visualization**: Charts showing price action, signals, and performance
- **Bias Estimation**: Statistical analysis to estimate overfitting and expected out-of-sample performance

## Installation

Ensure you have Rust installed, then build the project:

```bash
cargo build --release
```

## Usage

The program operates in two modes: optimization and prediction.

### Command Line Interface

```bash
try_diff_ev <COMMAND> [OPTIONS]
```

### Commands

#### Optimize Mode

Find optimal trading parameters using differential evolution:

```bash
try_diff_ev optimize [OPTIONS] <DATA_FILE>
```

**Parameters:**
- `--data-file <FILE>`: Path to market data file (required)
- `--max-lookback <N>`: Maximum lookback period for long MA (default: 6)
- `--max-thresh <F>`: Maximum threshold value ×10000 (default: 57.8112)
- `--popsize <N>`: Population size for differential evolution (default: 300)
- `--max-gens <N>`: Maximum generations to run (default: 10000)
- `--min-trades <N>`: Minimum trades required for valid solution (default: 20)
- `--train-pct <F>`: Training data percentage (0.0-1.0) (default: 0.7)
- `--output <FILE>`: Output filename for parameters (default: "params.txt")
- `--generator <TYPE>`: Signal generator type: "original" or "log_diff" (default: "original")
- `--output-dir <DIR>`: Output directory (default: "results/")
- `--verbose`: Enable verbose output

#### Predict Mode

Generate signals and backtest using optimized parameters:

```bash
try_diff_ev predict [OPTIONS] <DATA_FILE>
```

**Parameters:**
- `--data-file <FILE>`: Path to market data file (required)
- `--params-file <FILE>`: Path to optimized parameters file (default: "results/params.txt")
- `--budget <F>`: Initial trading budget (default: 10000.0)
- `--transaction-cost <F>`: Transaction cost percentage (default: 0.1)
- `--train-pct <F>`: Training data percentage for OOS testing (default: 0.7)
- `--output-dir <DIR>`: Output directory (default: "results/")
- `--generator <TYPE>`: Signal generator type: "original" or "log_diff" (default: "original")
- `--verbose`: Enable verbose output

## Parameter Meanings

### Trading Parameters (4 parameters optimized)

1. **Long Lookback Period** (`long_lookback`)
   - The number of periods for the long-term moving average
   - Range: 2 to `max_lookback`
   - Higher values = more smoothing, slower signals

2. **Short Percentage** (`short_pct`)
   - Percentage of long lookback used for short MA (0-99)
   - Determines short MA length as: `floor(long_lookback * short_pct / 100)`
   - Lower values = more responsive short MA

3. **Short Threshold** (`short_thresh`)
   - Threshold for SELL signals (in ×10000 format)
   - Example: 30.1 means 0.00301 (0.301%)
   - Smaller values = more sensitive SELL signals

4. **Long Threshold** (`long_thresh`)
   - Threshold for BUY signals (in ×10000 format)
   - Example: 57.8 means 0.00578 (0.578%)
   - Smaller values = more sensitive BUY signals

### Optimization Parameters

- **Population Size** (`popsize`): Number of candidate solutions in each generation
- **Maximum Generations** (`max_gens`): Maximum iterations of the evolutionary algorithm
- **Minimum Trades** (`min_trades`): Minimum number of trades required for a solution to be considered valid
- **Training Percentage** (`train_pct`): Fraction of data used for training (rest for out-of-sample testing)

### Signal Generators

#### Original (`original`)
- Uses ratio: `short_ma / long_ma - 1.0`
- BUY when ratio > long_threshold
- SELL when ratio < -short_threshold

#### Enhanced (`log_diff` or `enhanced`)
- Uses difference: `short_ma - long_ma`
- BUY when difference > long_threshold
- SELL when difference < -short_threshold
- Generally preferred for better signal characteristics

## Data Format

### Market Data File
The input data file should contain OHLC or price data in the following format:

```
YYYYMMDD Open High Low Close
YYYYMMDD Open High Low Close
...
```

**Requirements:**
- Space-separated values
- First column: Date in YYYYMMDD format
- Last column: Closing price (used for analysis)
- Prices must be positive numbers
- One data point per line

**Example:**
```
20200101 100.0 105.0 95.0 102.5
20200102 102.5 108.0 100.0 106.0
20200103 106.0 110.0 103.0 108.5
```

### Parameters File
Output from optimization, 4 lines of floating-point numbers:

```
6.0          # Long lookback
57.8         # Short percentage
30.1         # Short threshold
0.0          # Long threshold
```

## Output Files

### Optimization Mode
- `results/params.txt`: Optimized parameters
- `results/SENS.LOG`: Sensitivity analysis results
- Console output: Best fitness, parameter values, bias estimates

### Prediction Mode
- `results/trade_log.txt`: Detailed trade-by-trade log
- `results/signal_chart.png`: Price chart with signals and performance
- Console output: Backtest statistics and performance metrics

## Performance Metrics

### Backtest Results
- **ROI %**: Total return on investment
- **Total Trades**: Number of completed trades
- **Win Rate %**: Percentage of profitable trades
- **Max Drawdown %**: Maximum peak-to-trough decline
- **Sharpe Ratio**: Risk-adjusted return measure
- **Total Costs**: Cumulative transaction costs

### Bias Estimation
- **In-sample**: Performance on training data
- **Out-of-sample**: Performance on unseen data
- **Bias**: Estimated overfitting penalty
- **Expected**: Expected real-world performance

## Example Usage

### 1. Optimize Parameters
```bash
try_diff_ev optimize --data-file ../data/XAGUSD.txt --max-lookback 30 --popsize 200 --max-gens 5000 --generator enhanced --verbose
```

### 2. Backtest with Optimized Parameters
```bash
try_diff_ev predict --data-file ../data/XAGUSD.txt --budget 100000 --transaction-cost 0.2 --generator enhanced --verbose
```

### 3. Full Workflow
```bash
# Optimize on training data
try_diff_ev optimize ../data/market_data.txt --train-pct 0.6 --output optimized_params.txt

# Backtest on full dataset
try_diff_ev predict ../data/market_data.txt --params-file results/optimized_params.txt --budget 50000
```

## Signal Generation Logic

The system generates trading signals based on moving average crossovers:

1. **Calculate Moving Averages**:
   - Long MA: Simple moving average over `long_lookback` periods
   - Short MA: Simple moving average over `short_lookback` periods

2. **Generate Signal**:
   - Compare short MA vs long MA using selected algorithm
   - Apply thresholds to determine BUY/SELL/HOLD

3. **Position Management**:
   - BUY signal: Enter long position (or reverse from short)
   - SELL signal: Enter short position (or reverse from long)
   - HOLD: Maintain current position

## Risk Management

- **Transaction Costs**: Applied as percentage of portfolio value
- **Position Sizing**: Full portfolio allocation to each position
- **No Stop Losses**: System relies on signal-based exits
- **Drawdown Tracking**: Maximum drawdown calculated and reported

## Troubleshooting

### Common Issues

1. **"Insufficient data"**: Ensure your data file has enough price points (> max_lookback)
2. **"No valid price data"**: Check data format and ensure positive prices
3. **Poor optimization results**: Try increasing population size or generations
4. **Low trade count**: Reduce `min_trades` or adjust thresholds

### Parameter Tuning Tips

- Start with `max_lookback` = 10-30 for most markets
- Use `train_pct` = 0.6-0.8 for sufficient out-of-sample testing
- `popsize` = 100-500, `max_gens` = 1000-10000 depending on time constraints
- Lower thresholds = more frequent trading, higher thresholds = fewer but stronger signals

## Dependencies

- `statn`: Core statistical and financial libraries
- `plotters`: Chart generation for visualization
- `clap`: Command-line argument parsing
- `serde`: Serialization for data handling

## License

See the main project LICENSE file for licensing information.