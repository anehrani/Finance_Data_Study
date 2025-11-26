# try_cd_ma

A Rust program for selecting optimal moving average (MA) crossover indicators for financial trading using coordinate descent regularization.

## Overview

This program implements a systematic approach to indicator selection for trading strategies. It generates multiple moving average crossover indicators with different lookback periods, trains a linear model using coordinate descent with elastic net regularization, and evaluates the model's performance through cross-validation and backtesting.

## Features

- **Indicator Generation**: Automatically generates MA crossover indicators with configurable lookback periods
- **Model Training**: Uses coordinate descent with elastic net regularization for sparse feature selection
- **Cross-Validation**: Implements k-fold cross-validation for model selection
- **Backtesting**: Evaluates strategy performance on out-of-sample data
- **Performance Metrics**: Calculates returns, win rate, drawdown, and Sharpe ratio

## Dependencies

The program depends on several local crates in the statn workspace:
- `matlib`: Matrix operations
- `stats`: Statistical functions
- `finance_tools`: Financial calculations
- `indicators`: Technical indicators
- `backtesting`: Backtesting framework

## Usage

### Basic Command

```bash
cargo run --bin try_cd_ma -- [OPTIONS] <DATA_FILE>
```

### Command Line Arguments

- `DATA_FILE`: Path to market data file (required)
- `--lookback-inc <N>`: Increment for long-term lookback periods (default: 6)
- `--n-long <N>`: Number of long-term lookbacks to test (default: 6)
- `--n-short <N>`: Number of short-term lookbacks to test (default: 3)
- `--alpha <F>`: Elastic net mixing parameter (0-1] (default: 0.5)
- `--output-path <PATH>`: Output directory path (default: "results/")
- `--n-test <N>`: Number of test cases (default: 100)
- `--n-folds <N>`: Number of cross-validation folds (default: 10)
- `--n-lambdas <N>`: Number of lambda values to test (default: 50)
- `--max-iterations <N>`: Maximum coordinate descent iterations (default: 1000)
- `--tolerance <F>`: Convergence tolerance (default: 1e-9)

### Example

```bash
cargo run --bin try_cd_ma -- 2 3 2 0.5 ../data/XAGUSD.txt
```

This command:
- Uses lookback increment of 2
- Tests 30 long-term lookbacks
- Tests 10 short-term lookbacks per long-term period
- Uses alpha = 0.5 for elastic net
- Uses XAGUSD data file

## Data Format

The input data file should contain market data in the format:
```
YYYYMMDD Price
```

Where:
- `YYYYMMDD`: Date in YYYYMMDD format
- `Price`: Closing price as a floating-point number

Example:
```
20200101 100.0
20200102 101.5
20200103 99.8
```

## Output Files

The program generates several output files in the specified output directory:

- `model.json`: Trained model parameters in JSON format
- `CD_MA.LOG`: Detailed results log with model performance metrics
- `backtest_results.txt`: Backtest performance summary

## Model Details

### Indicators
The program generates moving average crossover indicators where each indicator is the difference between short-term and long-term simple moving averages.

### Training
- Uses coordinate descent algorithm with elastic net regularization
- Performs k-fold cross-validation to select optimal regularization parameter
- Targets are future price returns (logarithmic)

### Evaluation
- In-sample explained variance
- Out-of-sample total return
- Backtest metrics: ROI, win rate, max drawdown, Sharpe ratio

## Backtesting

The backtest uses:
- Initial capital: $100,000
- Transaction cost: 0.1% per trade
- Long/short positions based on model predictions

## Requirements

- Rust 2024 edition
- Access to the statn workspace crates
- Market data file in the specified format

## Troubleshooting

- Ensure sufficient training data: need at least `n_vars + 10` training cases
- Check data file format and path
- Verify alpha parameter is in range (0, 1]
- Ensure output directory exists and is writable