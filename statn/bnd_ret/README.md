# bnd_ret - Bounding Future Returns

A Rust implementation of a primitive moving-average-crossover system that demonstrates bounding future returns using order statistics.

## Overview

This tool performs walk-forward analysis on market data using a simple moving average crossover strategy. It then computes statistical bounds on future returns based on the out-of-sample performance.

## Features

- **Moving Average Crossover System**: Optimizes short-term and long-term lookback periods
- **Walk-Forward Analysis**: Tests the system on successive time periods
- **Return Bounding**: Calculates lower and upper bounds on future returns with configurable confidence levels
- **Order Statistics**: Uses order statistics to provide probabilistic bounds

## Building

```bash
cargo build --release -p bnd_ret
```

## Usage

```bash
cargo run --release -p bnd_ret -- <max_lookback> <n_train> <n_test> <lower_fail> <upper_fail> <p_of_q> <filename>
```

### Arguments

- `max_lookback`: Maximum moving-average lookback period to test
- `n_train`: Number of bars in the training set (should be much greater than max_lookback)
- `n_test`: Number of bars in the test set
- `lower_fail`: Lower bound failure rate (typically 0.01-0.1)
- `upper_fail`: Upper bound failure rate (typically 0.1-0.5)
- `p_of_q`: Probability of bad bound (typically 0.01-0.1)
- `filename`: Path to market data file

### Market Data Format

The market data file should be a text file with the following format:
```
YYYYMMDD Price
```

Example:
```
20200101 3230.78
20200102 3257.85
20200103 3234.85
```

## Example

```bash
cargo run --release -p bnd_ret -- 100 1000 63 0.1 0.4 0.05 market_data.txt
```

This will:
- Test lookback periods up to 100 bars
- Use 1000 bars for training
- Test on 63-bar periods
- Set lower bound failure rate to 10%
- Set upper bound failure rate to 40%
- Use 5% probability for bad bounds
- Read data from `market_data.txt`

## Output

The program outputs:
- In-sample (IS) and out-of-sample (OOS) performance for each walk-forward period
- Mean OOS return across all periods
- Lower and upper bounds on future returns
- Probabilistic interpretations of the bounds (optimistic and pessimistic views)

## Statistical Background

The tool uses order statistics to compute bounds on future returns. For a given failure rate, it calculates:
- The quantile of the return distribution
- Confidence intervals for the true failure rate
- Optimistic and pessimistic interpretations of the bounds

## Testing

Run the test suite:

```bash
cargo test -p bnd_ret
```

## License

This is part of the statn project.
