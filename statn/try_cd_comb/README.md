# try_cd_comb - Combined Indicator Selection using Coordinate Descent

This package extends the `try_cd_ma` (Moving Average Crossover) system by supporting a **combination of multiple indicator types**, including:
- **Moving Average (MA) Crossovers** - Trend-following indicators
- **Relative Strength Index (RSI)** - Momentum oscillator
- **MACD (Moving Average Convergence Divergence)** - Trend and momentum indicator

The system uses **Elastic Net regularization** via coordinate descent to select the most predictive indicators for trading.

## Overview

The `try_cd_comb` package performs the following:

1. **Generates multiple indicator specifications** based on configuration:
   - MA crossovers with varying short/long lookback periods
   - RSI with configurable periods (e.g., 14, 21, 28)

2. **Computes indicators** for both training and test datasets

3. **Trains a linear model** using coordinate descent with elastic net regularization:
   - Cross-validation to select optimal lambda (regularization strength)
   - Sparse coefficient selection (many indicators will have zero weight)

4. **Evaluates out-of-sample performance** on test data

5. **Generates detailed reports** showing:
   - Selected indicators and their coefficients
   - Cross-validation results
   - Out-of-sample returns

## Configuration

### TOML Configuration File

Create a configuration file (e.g., `config.toml`):

```toml
# Increment to long-term lookback
lookback_inc = 10

# Number of long-term lookbacks to test
n_long = 20

# Number of short-term lookbacks to test
n_short = 10

# Alpha parameter for elastic net (0-1]
# 0 = Ridge, 1 = Lasso
alpha = 0.5

# Path to market data file (YYYYMMDD Price format)
data_file = "data/XAGUSD.txt"

# Path to output results file
output_file = "CD_COMB.LOG"

# Number of test cases (default: 252 = one year)
n_test = 252

# Number of cross-validation folds
n_folds = 10

# Number of lambda values to test
n_lambdas = 50

# Maximum iterations for coordinate descent
max_iterations = 1000

# Convergence tolerance
tolerance = 1e-9

# RSI periods to include (optional)
rsi_periods = [14, 21, 28]
```

### Command-Line Arguments

Run with a config file:
```bash
cargo run -p try_cd_comb -- --config config.toml
```

Or provide arguments directly:
```bash
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt --rsi-periods 14,21,28
```

Arguments:
- `LOOKBACK_INC`: Increment to long-term lookback
- `N_LONG`: Number of long-term lookbacks
- `N_SHORT`: Number of short-term lookbacks
- `ALPHA`: Alpha parameter (0-1]
- `FILENAME`: Market data file
- `--rsi-periods`: Comma-separated RSI periods (optional)

## How It Works

### 1. Indicator Generation

**MA Crossovers**: For each combination of long and short lookback periods:
- Short MA - Long MA = crossover signal
- Positive values suggest uptrend, negative suggest downtrend

**RSI**: For each specified period:
- Measures momentum on a 0-100 scale
- Values > 70 suggest overbought, < 30 suggest oversold

**MACD**: Uses standard parameters (12, 26, 9):
- Fast EMA (12) - Slow EMA (26) = MACD line
- Signal line = EMA(9) of MACD line
- Histogram = MACD line - Signal line (used as the indicator)
- Positive histogram suggests bullish momentum, negative suggests bearish

### 2. Model Training

Uses **Elastic Net** regularization:
```
minimize: (1/2n) * ||y - Xβ||² + λ[(1-α)/2 * ||β||² + α * ||β||₁]
```

Where:
- `y` = target returns
- `X` = indicator matrix
- `β` = coefficients to learn
- `λ` = regularization strength (selected via CV)
- `α` = elastic net mixing parameter

**Cross-validation** selects the optimal `λ` that maximizes out-of-sample explained variance.

### 3. Trading Logic

For each time step:
1. Compute all indicators
2. Calculate prediction: `pred = Σ(β_i * indicator_i)`
3. Trade based on prediction:
   - If `pred > 0`: Go long (buy)
   - If `pred < 0`: Go short (sell)
   - If `pred = 0`: No position

### 4. Evaluation

Reports:
- **In-sample explained variance**: How well the model fits training data
- **Out-of-sample return**: Cumulative log return on test data
- **Selected indicators**: Non-zero coefficients indicate useful indicators

## Output

The program generates a log file (e.g., `CD_COMB.LOG`) containing:

```
CD_MA - Moving Average Crossover Indicator Selection
============================================================

Configuration:
  Lookback increment: 10
  Number of long-term lookbacks: 20
  Number of short-term lookbacks: 10
  Alpha: 0.5000
  Number of indicators: 203
  Test cases: 252

Cross-Validation Results:
  Optimal lambda: 0.012345

  Lambda    OOS Explained
  ---------------------------
  0.0001         0.1234
  0.0002         0.1456
  ...

Beta Coefficients (In-sample explained variance: 45.2%):
Row: long-term lookback | Columns: short-term lookback (small to large)

   10    0.1234    0.0000    ----    ...
   20    ----      0.2345    ----    ...
   ...

RSI Coefficients:
  Period  14:    0.0567
  Period  21:      ----
  Period  28:   -0.0234

MACD Coefficient (Histogram):
  MACD:    0.0123

Out-of-Sample Results:
  Total return: 0.12345 (13.145%)
```

A detailed backtest report is also generated in `backtest_results.txt` (or similar), containing performance metrics and a full trade log.

## Data Format

Input data should be in the format:
```
YYYYMMDD Price
20200101 1234.56
20200102 1235.67
...
```

## Example Usage

```bash
# Using config file
cargo run -p try_cd_comb -- --config config.toml

# Using command-line arguments with RSI and MACD
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt --rsi-periods 14,21 --include-macd

# MA only (no RSI, no MACD)
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt
```

## Testing

Run tests:
```bash
cargo test -p try_cd_comb
```

## Architecture

The package is organized into modules:

- **`config.rs`**: Configuration parsing and validation
- **`data.rs`**: Market data loading and train/test splitting
- **`indicators.rs`**: Indicator specification and computation
- **`training.rs`**: Model training with cross-validation
- **`evaluation.rs`**: Model evaluation and result reporting
- **`main.rs`**: Entry point and workflow orchestration

## Future Enhancements

Potential additions:
- **Bollinger Bands** (volatility indicator)
- **Stochastic Oscillator** (momentum indicator)
- **ATR** (Average True Range - volatility)
- **Volume-based indicators**
- **Custom MACD parameters** (currently uses standard 12, 26, 9)

## References

- Coordinate Descent: Friedman, J., Hastie, T., & Tibshirani, R. (2010). Regularization Paths for Generalized Linear Models via Coordinate Descent.
- Elastic Net: Zou, H., & Hastie, T. (2005). Regularization and variable selection via the elastic net.
