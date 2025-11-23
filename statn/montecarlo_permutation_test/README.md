# Monte Carlo Permutation Test (MCPT)

A Rust implementation of Monte Carlo Permutation Testing for trading systems. This package provides two distinct testing modes for evaluating trading strategies and estimating their true skill.

## Overview

Monte Carlo Permutation Testing is a statistical technique used to:
1. Test for outstanding performance with unpermuted data (simultaneously tests for excessive weakness and excessive strength/overfitting)
2. Estimate true skill and unbiased future return by accounting for training bias

## Features

### Two Testing Modes

#### 1. Bars Mode (Mean Reversion System)
Tests a primitive mean reversion long-only system using OHLC bar data.

**Strategy**: 
- Goes long when there's a long-term rise followed by a short-term drop
- Optimizes rise and drop thresholds
- Uses next-open-to-open returns

**Command**:
```bash
cargo run --release --bin mcpt bars <lookback> <nreps> <filename>
```

**Arguments**:
- `lookback`: Long-term rise lookback period
- `nreps`: Number of MCPT replications (hundreds or thousands recommended)
- `filename`: Path to market data file (YYYYMMDD Open High Low Close format)

**Example**:
```bash
cargo run --release --bin mcpt bars 300 1000 data/market_ohlc.txt
```

#### 2. Trend Mode (Moving Average Crossover)
Tests a primitive moving-average crossover system using single price series.

**Strategy**:
- Goes long when short-term MA > long-term MA
- Goes short when short-term MA < long-term MA
- Optimizes both lookback periods

**Command**:
```bash
cargo run --release --bin mcpt trend <max_lookback> <nreps> <filename>
```

**Arguments**:
- `max_lookback`: Maximum moving-average lookback to test
- `nreps`: Number of MCPT replications (hundreds or thousands recommended)
- `filename`: Path to market data file (YYYYMMDD Price format)

**Example**:
```bash
cargo run --release --bin mcpt trend 300 1000 data/market_prices.txt
```

## Input File Formats

### OHLC Format (Bars Mode)
```
YYYYMMDD Open High Low Close
20200101 100.0 102.0 99.0 101.0
20200102 101.0 103.0 100.5 102.5
```

### Price Format (Trend Mode)
```
YYYYMMDD Price
20200101 100.0
20200102 101.5
20200103 99.8
```

**Notes**:
- Date must be 8 digits (YYYYMMDD)
- Prices can be separated by spaces, tabs, or commas
- All prices must be positive (automatically converted to log prices internally)

## Output Metrics

The program outputs the following key metrics:

- **p-value**: Probability that the system's performance is due to chance (null hypothesis: system is worthless)
- **Original return**: Total return from the unpermuted data
- **Trend component**: Portion of return attributable to market trend
- **Training bias**: Overfitting bias from parameter optimization
- **Skill**: True skill after removing trend and bias
- **Unbiased return**: Expected future return (original - training bias)

### Interpreting Results

- **Low p-value (< 0.05)**: System shows statistically significant performance
- **High p-value (> 0.05)**: Performance likely due to chance
- **Positive skill**: System has genuine predictive ability beyond trend-following
- **Negative skill**: System underperforms even after accounting for bias

## Building

```bash
# Build in release mode (recommended for performance)
cargo build --release

# Run tests
cargo test
```

## How It Works

### Permutation Process

1. **Preparation**: Compute price changes from the original data
2. **Permutation**: Shuffle price changes randomly while preserving:
   - Starting price
   - Ending price
   - Statistical properties of returns
3. **Optimization**: Find optimal parameters for each permuted series
4. **Statistical Analysis**: Compare original performance to permuted distribution

### Key Concepts

**Training Bias**: The tendency for optimized parameters to overfit historical data. MCPT estimates this bias by comparing performance on the original data to performance on permuted data with the same statistical properties.

**Trend Component**: The portion of returns attributable to the overall market trend. Calculated based on the number of long/short positions and the average trend per return.

**Skill**: The true predictive ability of the system after removing both trend effects and training bias.

## Technical Details

### Random Number Generator
Uses the MWC256 (Multiply-With-Carry) generator suggested by Marsaglia in his DIEHARD suite, providing excellent speed and quality.

### Permutation Strategy
- **Bars mode**: Preserves open-to-open price differences at endpoints
- **Trend mode**: Preserves first and last prices
- Both modes maintain the statistical distribution of returns

### Optimization
- **Bars mode**: Grid search over 50×50 combinations of rise/drop thresholds
- **Trend mode**: Grid search over all valid short/long lookback combinations

## Performance Considerations

- Use release builds for production runs (`--release` flag)
- Larger `nreps` values provide more reliable statistics but take longer
- Typical runs: 1000-10000 replications
- Processing time scales linearly with `nreps`

## References

This implementation is based on Monte Carlo permutation testing techniques for evaluating trading systems, accounting for:
- Selection bias from parameter optimization
- Market trend effects
- Statistical significance of results

## License

Part of the Hilbert statistical analysis toolkit.
