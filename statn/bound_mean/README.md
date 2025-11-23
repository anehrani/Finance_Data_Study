# bound_mean

`bound_mean` is a Rust package converted from C++ that compares methods for bounding expected returns using bootstrap confidence intervals.

## Usage

```bash
cargo run -p bound_mean -- <max_lookback> <n_train> <n_test> <n_boot> <filename>
```

Arguments:
- `max_lookback`: Maximum moving-average lookback
- `n_train`: Number of bars in training set (must be at least 10 greater than max_lookback)
- `n_test`: Number of bars in test set
- `n_boot`: Number of bootstrap replications
- `filename`: Path to market file (Format: YYYYMMDD Price)

## Example

```bash
cargo run -p bound_mean -- 100 2000 1000 1000 path/to/market_data.txt
```

## Description

The tool performs a walk-forward analysis on market data using a moving average breakout system. It computes returns for:
1.  All bars (Grouped)
2.  Bars with open positions (Open posn)
3.  Completed trades (Complete)

It then calculates 90% lower confidence bounds using:
-   Student's t
-   Percentile method
-   Pivot method
-   BCa (Bias-Corrected and Accelerated) bootstrap

## Modules

-   `main.rs`: Main logic, walk-forward loop, and return calculation.
-   `boot_conf.rs`: Bootstrap confidence interval implementations (Percentile, BCa).
-   `stats.rs`: Statistical functions (CDF, inverse CDF).
-   `unifrand.rs`: Random number generation wrapper.
-   `qsort.rs`: Sorting utilities (placeholder).
