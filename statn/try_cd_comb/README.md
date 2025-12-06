# try_cd_comb - Combined Indicator Selection using Coordinate Descent

This package extends the `try_cd_ma` (Moving Average Crossover) system by supporting a flexible combination of multiple indicator types, including:
- **Moving Average (MA)** - Simple Moving Average crossovers
- **Relative Strength Index (RSI)** - Momentum oscillator differences
- **Exponential Moving Average (EMA)** - EMA crossovers
- **MACD** - MACD Histogram values
- **Rate of Change (ROC)** - Momentum indicator differences

The system uses **Elastic Net regularization** via coordinate descent to select the most predictive linear combination of these indicators.

## Overview

The `try_cd_comb` package performs the following:

1. **Generates multiscale indicator specifications**:
   - Instead of fixed parameters, it generates a grid of indicators based on `n_long` (number of long-term periods), `n_short` (number of short-term periods), and `lookback_inc` (increment).
   - This applies to ALL enabled indicator types. For example, if you enable MACD, it will generate many MACD indicators with different fast/slow periods derived from the grid.

2. **Computes indicators** for both training and test datasets.

3. **Trains a linear model** using coordinate descent with elastic net regularization:
   - Cross-validation to select optimal lambda (regularization strength).
   - Sparse coefficient selection (many indicators will have zero weight).

4. **Evaluates out-of-sample performance** on test data.

5. **Generates detailed reports** showing coefficients for each indicator type.

## Configuration

### TOML Configuration File

Create a configuration file (e.g., `config.toml`):

```toml
# Increment to long-term lookback (e.g., 10 means 10, 20, 30...)
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

# Crossover types to generate
# Options: "ma", "rsi", "ema", "macd", "roc"
crossover_types = ["ma", "rsi", "macd"]
```

### Command-Line Arguments

Run with a config file:
```bash
cargo run -p try_cd_comb -- --config config.toml
```

Or provide arguments directly:
```bash
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt --crossover-types ma,macd,roc
```

### Argument Meanings

- `LOOKBACK_INC`: The step size for generating long-term lookback periods. (e.g., if 10, periods are 10, 20, 30...)
- `N_LONG`: The number of long-term lookback periods to generate. (Max lookback = `N_LONG` * `LOOKBACK_INC`)
- `N_SHORT`: The number of short-term lookback periods to generate for each long-term period. Short lookbacks are derived as a fraction of the long lookback to ensures they are always shorter.
- `ALPHA`: Elastic Net mixing parameter. `1.0` is Lasso (L1 penalty), `0.0` is Ridge (L2 penalty). `0.5` is a mix.
- `FILENAME`: Path to the input market data file.
- `--crossover-types`: Comma-separated list of indicator types to include.
    - `ma`: Simple Moving Average Crossover (`SMA(short) - SMA(long)`)
    - `rsi`: RSI Crossover (`RSI(short) - RSI(long)`)
    - `ema`: Exponential Moving Average Crossover (`EMA(short) - EMA(long)`)
    - `macd`: MACD Histogram (`MACD(fast=short, slow=long, signal=9)`)
    - `roc`: Rate of Change Crossover (`ROC(short) - ROC(long)`)

## How It Works

### Unified Indicator Generation

All enabled indicator types are generated using the same grid search logic:

1. **Long Lookback Loop**: `ilong` ranges from 0 to `n_long`.
   `long_lookback = (ilong + 1) * lookback_inc`

2. **Short Lookback Loop**: `ishort` ranges from 0 to `n_short`.
   `short_lookback = long_lookback * (ishort + 1) / (n_short + 1)`

3. **Indicator Computation**:
   - **MA/EMA/RSI/ROC**: Computes `Indicator(short) - Indicator(long)`. A positive value usually indicates an uptrend (short > long).
   - **MACD**: Computes the MACD Histogram where `fast_period = short_lookback`, `slow_period = long_lookback`, and `signal_period` is fixed at 9.

This approach creates a vast "feature soup" of indicators at multiple time scales. The Elastic Net model then statistically selects the best combination of these features to predict returns.

## Example Output

The program generates a log file (e.g., `CD_COMB.LOG`) containing coefficients for each type:

```
MA Crossover Coefficients:
Row: long-term lookback | Columns: short-term lookback
   10    0.1234    0.0000    ----    ...
   20    ----      0.2345    ----    ...

MACD Crossover Coefficients:
Row: long-term lookback | Columns: short-term lookback
   10    0.0567    ----      ----    ...
   ...
```

## Data Format

Input data should be in the format:
```
YYYYMMDD Price
20200101 1234.56
20200102 1235.67
...
```

## Testing

Run tests:
```bash
cargo test -p try_cd_comb
```
