# statn Statistical Utility Tools

The `statn` workspace includes several standalone binaries for specialized statistical analysis.

## Tools Overview

### Stationarity and Information
- **`stationary_test`**: Checks if a time series is stationary (e.g., using ADF test). Stationary data is often a prerequisite for reliable modeling.
- **`check_entropy`**: Computes the Shannon entropy of the data to estimate its "randomness" or information density.

### Validation and Bias
- **`montecarlo_permutation_test` (MCPT)**: Uses noise-shuffling techniques to determine if a strategy's performance could have been achieved by chance.
- **`train_bias`**: Specifically designed to measure and correct for training/selection bias.
- **`cross_validation_mkt`**: Market-aware cross-validation to ensure models generalize across different market regimes.

### Risk and Distribution
- **`drawdown`**: Analyzes the theoretical and empirical drawdown characteristics of a strategy.
- **`bound_mean` / `bootstrap_rate`**: Bootstrap-based tools for estimating confidence intervals for means and success rates.
- **`conftest`**: Performs "confidence testing" on model outcomes.

## General Usage
Most of these tools are run via `cargo run --bin <tool_name>`. They typically expect a data file and some numeric parameters. Refer to the tool's source code (usually `main.rs` in its directory) for specific argument order.
