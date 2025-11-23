# bootstrap_rate

This package implements bootstrap confidence intervals for Profit Factor and Sharpe Ratio, converted from the original C++ implementation.

## Usage

```bash
cargo run -p bootstrap_rate -- <nsamples> <nboot> <ntries> <prob>
```

Arguments:
- `nsamples`: Number of price changes in market history (e.g., 1000)
- `nboot`: Number of bootstrap replications (e.g., 1000)
- `ntries`: Number of trials for generating summary (e.g., 100)
- `prob`: Probability that a trade will be a win (e.g., 0.6)

## Example

```bash
cargo run -p bootstrap_rate -- 1000 100 100 0.6
```

## Methods

The package implements two bootstrap methods for confidence intervals:
1.  **Percentile Method**: Uses the percentiles of the bootstrap distribution.
2.  **BCa (Bias-Corrected and accelerated) Method**: Adjusts for bias and skewness in the bootstrap distribution.

It also computes a "Pivot" interval derived from the percentile method.
