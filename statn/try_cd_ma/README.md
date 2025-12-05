# try_cd_ma

**Moving Average Crossover Indicator Selection using Coordinate Descent**

## Overview

`try_cd_ma` is a quantitative trading strategy tool that uses elastic net regularization (via coordinate descent) to select optimal moving average crossover indicators for predicting market returns. The algorithm generates multiple MA crossover signals with varying lookback periods and uses machine learning to identify which combinations are most predictive.

## Algorithm

### How It Works

1. **Indicator Generation**: Creates a grid of moving average crossover indicators by varying:
   - Long-term MA lookback periods (controlled by `n_long` and `lookback_inc`)
   - Short-term MA lookback periods (controlled by `n_short`)
   - Total indicators = `n_long × n_short`

2. **Feature Engineering**: For each time period, computes the difference between short and long moving averages as a trading signal

3. **Model Training**: Uses coordinate descent with elastic net regularization to:
   - Select the most predictive indicator combinations
   - Avoid overfitting through L1 (Lasso) and L2 (Ridge) penalties
   - Find optimal regularization strength via cross-validation

4. **Evaluation**: Tests the trained model on out-of-sample data to measure predictive performance

### Key Parameters

- **`alpha`**: Elastic net mixing parameter (0 < α ≤ 1)
  - α = 1.0: Pure Lasso (L1) - promotes sparsity
  - α = 0.5: Balanced elastic net
  - α → 0: Pure Ridge (L2) - shrinks coefficients

- **`lambda`**: Regularization strength (automatically selected via cross-validation)

## Installation

This package is part of the `statn` workspace. Ensure you have Rust installed, then navigate to the project directory:

```bash
cd /path/to/Hilbert_project_01/statn/try_cd_ma
```

## Usage

### Option 1: Using a Configuration File (Recommended)

1. **Create a configuration file** (see `config.example.toml`):

```toml
# config.toml
lookback_inc = 10
n_long = 20
n_short = 10
alpha = 0.5
data_file = "../../data/historical_data/XAGUSD.txt"
output_file = "results/CD_MA.LOG"
n_test = 252
n_folds = 10
n_lambdas = 50
max_iterations = 1000
tolerance = 1e-9
```

2. **Run with the config file**:

```bash
cargo run --release -- --config config.toml
```

### Option 2: Using Command-Line Arguments

```bash
cargo run --release -- <LOOKBACK_INC> <N_LONG> <N_SHORT> <ALPHA> <FILENAME>
```

**Example**:
```bash
cargo run --release -- 10 20 10 0.5 ../../data/historical_data/XAGUSD.txt
```

### Input Data Format

The market data file should be in the following format:
```
YYYYMMDD Price
20200101 1520.50
20200102 1525.30
20200103 1518.75
...
```

- **Date**: 8-digit format (YYYYMMDD)
- **Price**: Decimal number (close price)
- One entry per line, space-separated

## Configuration Parameters

| Parameter | Description | Default | Range |
|-----------|-------------|---------|-------|
| `lookback_inc` | Increment for long-term lookback periods | - | > 0 |
| `n_long` | Number of long-term lookback periods to test | - | > 0 |
| `n_short` | Number of short-term lookback periods to test | - | > 0 |
| `alpha` | Elastic net mixing parameter | - | (0, 1] |
| `data_file` | Path to market data file | - | Valid file path |
| `output_file` | Path to output results file | `CD_MA.LOG` | Valid file path |
| `n_test` | Number of out-of-sample test cases | 252 | > 0 |
| `n_folds` | Number of cross-validation folds | 10 | ≥ 2 |
| `n_lambdas` | Number of lambda values to test in CV | 50 | > 0 |
| `max_iterations` | Maximum iterations for coordinate descent | 1000 | > 0 |
| `tolerance` | Convergence tolerance | 1e-9 | > 0 |

## Output

The program generates:

1. **Console Output**: Real-time progress and summary statistics
   ```
   CD_MA - Moving Average Crossover Indicator Selection
   
   Loading market data...
   Training cases: 2500
   Test cases: 252
   Number of indicators: 200
   Computing training indicators...
   Running 10-fold cross-validation...
   Optimal lambda: 0.001234
   Training final model...
   In-sample explained variance: 15.234%
   Computing test indicators...
   
   Running backtest on test data...
   Backtest completed:
     Total trades: 45
     Total return: 12.34%
     Win rate: 55.56%
     Max drawdown: 8.21%
     Sharpe ratio: 1.234
   
   Summary:
     In-sample explained variance: 15.234%
     OOS total return: 0.08456 (8.456%)
   ```

2. **Results File** (default: `CD_MA.LOG`): Detailed results including:
   - Configuration parameters
   - Selected indicators and their coefficients
   - Cross-validation performance
   - In-sample and out-of-sample metrics

3. **Backtest Results File** (default: `backtest_results.txt`): Comprehensive backtesting analysis including:
   - Performance metrics (ROI, win rate, Sharpe ratio, max drawdown)
   - Detailed trade log with entry/exit prices and P&L
   - Equity curve summary

## Example Workflow

```bash
# 1. Navigate to the project directory
cd statn/try_cd_ma

# 2. Copy and customize the example config
cp config.example.toml my_config.toml
# Edit my_config.toml with your preferred settings

# 3. Run the analysis
cargo run --release -- --config my_config.toml

# 4. Review results
cat results/CD_MA.LOG
```

## Understanding the Results

### In-Sample Explained Variance
Percentage of return variance explained by the model on training data. Higher values indicate better fit, but watch for overfitting.

### Out-of-Sample (OOS) Return
The cumulative return achieved by the model on unseen test data. This is the key metric for evaluating real-world performance.

### Lambda Selection
The optimal lambda is chosen to maximize out-of-sample performance during cross-validation. Lower lambda = less regularization (more complex model).

## Tips for Best Results

1. **Start Conservative**: Begin with moderate values (e.g., `n_long=20`, `n_short=10`) to avoid overfitting

2. **Sufficient Data**: Ensure you have at least `n_vars + 10` training cases, where `n_vars = n_long × n_short`

3. **Alpha Selection**:
   - Use α = 1.0 for maximum sparsity (fewer selected indicators)
   - Use α = 0.5 for balanced regularization
   - Lower α if you want more indicators included

4. **Test Period**: Use `n_test = 252` for one trading year of out-of-sample testing

## Dependencies

- `serde` - Configuration serialization
- `toml` - TOML config file parsing
- `clap` - Command-line argument parsing
- `anyhow` - Error handling
- `matlib` - Matrix operations and coordinate descent
- `stats` - Statistical utilities
- `indicators` - Technical indicator calculations
- `finance_tools` - Financial data utilities
- `backtesting` - Backtesting framework for trading strategies
- `statn` - Core statistical and modeling utilities

## Project Structure

```
try_cd_ma/
├── Cargo.toml              # Package configuration
├── README.md               # This file
├── config.example.toml     # Example configuration
├── main.rs                 # Entry point
├── src/
│   ├── lib.rs             # Public API
│   ├── config.rs          # Configuration management
│   ├── data.rs            # Data loading and splitting
│   ├── indicators.rs      # MA crossover indicator generation
│   ├── training.rs        # Model training with CV
│   ├── evaluation.rs      # Model evaluation and reporting
│   └── backtest.rs        # Backtesting integration
└── results/               # Output directory
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with verbose output:

```bash
cargo test -- --nocapture
```

## Troubleshooting

### "Insufficient training data" Error
- **Cause**: Not enough data points for the number of indicators
- **Solution**: Reduce `n_long` or `n_short`, or use more historical data

### "Invalid open/high/low/close reading" Error
- **Cause**: Malformed data file
- **Solution**: Verify data file format (YYYYMMDD Price, space-separated)

### Poor Out-of-Sample Performance
- **Cause**: Overfitting or non-stationary market conditions
- **Solution**: 
  - Increase regularization (higher lambda, achieved by increasing alpha)
  - Reduce number of indicators
  - Use more training data
  - Check if market regime changed between train/test periods

## License

Part of the Hilbert Project statistical analysis toolkit.

## Related Projects

- `stationary_test` - Test for stationarity in time series
- `check_entropy` - Entropy analysis for market data
- `try_diff_ev` - Differential evolution optimization for trading strategies
