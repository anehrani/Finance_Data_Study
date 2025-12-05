# try_cd_ma

**Moving Average Crossover Indicator Selection using Coordinate Descent**

## Overview

`try_cd_ma` is a quantitative trading strategy tool that uses elastic net regularization (via coordinate descent) to select optimal moving average crossover indicators for predicting market returns.

## Usage

Run the tool using `cargo run`:

```bash
cargo run --release -- [OPTIONS] <DATA_FILE>
```

### Example

```bash
cargo run --release -- --lookback-inc 10 --n-long 20 --n-short 10 --alpha 0.5 ../../data/historical_data/^GSPC.txt
```

### Arguments

- `<DATA_FILE>`: Path to market data file (YYYYMMDD Price format)

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--lookback-inc` | Increment to long-term lookback | 2 |
| `--n-long` | Number of long-term lookbacks to test | 6 |
| `--n-short` | Number of short-term lookbacks to test | 5 |
| `--alpha` | Alpha parameter for elastic net (0-1] | 0.5 |
| `--output-path` | Path to output results directory | `results/` |
| `--n-test` | Number of test cases | 252 |
| `--n-folds` | Number of cross-validation folds | 10 |
| `--n-lambdas` | Number of lambda values to test | 50 |
| `--max-iterations` | Maximum iterations | 1000 |
| `--tolerance` | Convergence tolerance | 1e-9 |

## Input Data Format

The market data file should be in the following format (space-separated):

```
YYYYMMDD Price
20200101 1520.50
20200102 1525.30
...
```

## Output

The program generates results in the specified `--output-path` (default: `results/`):

1. **Console Output**: Real-time progress and summary statistics.
2. **`CD_MA.LOG`**: Detailed results including selected indicators and model metrics.
3. **`backtest_results.txt`**: Comprehensive backtesting analysis including ROI, Sharpe ratio, and trade log.

## Troubleshooting

### "Insufficient training data" Error
- **Cause**: Not enough data points for the number of indicators.
- **Solution**: Reduce `n_long` or `n_short`, or use a larger dataset.

### "Invalid open/high/low/close reading" Error
- **Cause**: Malformed data file.
- **Solution**: Verify data file format (YYYYMMDD Price, space-separated).
