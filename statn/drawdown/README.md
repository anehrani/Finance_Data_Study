# Drawdown Analysis

A Rust implementation for testing methods to find percentiles of future drawdown in trading systems.

## Overview

This package implements and compares two methods for estimating drawdown quantiles:

1. **Incorrect Method**: Bootstrap resampling from a fixed sample of trades
2. **Correct Method**: Bootstrap resampling from the full history, then computing drawdown quantiles for each bootstrap sample

The program performs Monte Carlo simulation to evaluate how well each method predicts actual population drawdown percentiles.

## Background

Drawdown is the maximum loss from a peak to a subsequent trough in a trading equity curve. Understanding the distribution of potential future drawdowns is critical for risk management. However, naively bootstrapping from a sample of trades can lead to biased estimates of drawdown quantiles.

This program demonstrates that:
- Simply bootstrapping trades and computing drawdown percentiles underestimates the true risk
- The correct approach is to bootstrap full equity curves and compute drawdown quantiles for each curve

## Installation

This package is part of the `statn` workspace. To build:

```bash
cargo build --release -p drawdown
```

## Usage

```bash
cargo run --release -p drawdown -- <Nchanges> <Ntrades> <WinProb> <BoundConf> <BootstrapReps> <QuantileReps> <TestReps>
```

### Parameters

- `Nchanges`: Number of price changes in the historical sample (e.g., 252 for one year of daily data)
- `Ntrades`: Number of trades to analyze for drawdown (e.g., 252)
- `WinProb`: Probability of a winning trade (0.0 to 1.0, e.g., 0.5 for 50% win rate)
- `BoundConf`: Confidence level for the correct method bounds (typically 0.5 to 0.999, e.g., 0.8)
- `BootstrapReps`: Number of bootstrap replications (e.g., 1000)
- `QuantileReps`: Number of bootstrap replications for finding drawdown quantiles (e.g., 1000)
- `TestReps`: Number of test replications for the study (e.g., 100)

### Example

```bash
cargo run --release -p drawdown -- 252 252 0.5 0.8 1000 1000 100
```

This runs a simulation with:
- 252 historical price changes
- 252 trades per drawdown period
- 50% win probability
- 80% confidence bounds
- 1000 bootstrap replications
- 1000 quantile bootstrap replications
- 100 test replications

## Output

The program writes results to `DRAWDOWN.LOG` and displays progress on the console.

### Console Output

For each test iteration, the program displays:

```
Mean return
  Actual    Incorrect
   0.001   0.00234
   0.01    0.01456
   0.05    0.05123
   0.1     0.10234

Drawdown
  Actual    Incorrect  Correct
   0.001   0.00456    0.00102
   0.01    0.02345    0.01023
   0.05    0.08234    0.05012
   0.1     0.14567    0.10123
```

The columns show:
- **Actual**: The nominal percentile (e.g., 0.001 = 0.1%)
- **Incorrect**: Observed frequency using the incorrect bootstrap method
- **Correct**: Observed frequency using the correct bootstrap method

Ideally, the observed frequencies should match the actual percentiles. The incorrect method typically overestimates the frequency of extreme drawdowns (showing the method is too optimistic), while the correct method should be well-calibrated.

### Log File

The `DRAWDOWN.LOG` file contains:
- Input parameters
- Detailed results with ratios showing how far off each method is from the nominal percentile

## Implementation Details

### Random Number Generation

The package uses Marsaglia's MWC256 (Multiply-With-Carry) algorithm for random number generation, matching the original C++ implementation. This ensures:
- High-quality random numbers
- Deterministic results with the same seed
- Fast generation

### Normal Distribution

Standard normal random variables are generated using the Box-Muller transform:
```
X = sqrt(-2 * ln(U1)) * cos(2π * U2)
```
where U1 and U2 are uniform random variables.

### Drawdown Calculation

Drawdown is computed as:
```
DD = max(Peak - Current)
```
where Peak is the maximum cumulative return seen so far.

## Module Structure

- `src/random.rs`: Random number generation (MWC256 RNG, Box-Muller normal)
- `src/drawdown.rs`: Core drawdown calculation functions
- `src/lib.rs`: Library exports
- `src/main.rs`: Command-line application

## Testing

Run the test suite:

```bash
cargo test -p drawdown
```

The test suite includes:
- RNG determinism tests
- Uniform distribution range tests
- Normal distribution statistical tests
- Drawdown calculation tests
- Quantile calculation tests

## Theory

### The Problem with Naive Bootstrap

When you bootstrap from a sample of trades and compute drawdown, you're implicitly assuming the trades are independent. However, drawdown is a path-dependent statistic that depends on the sequence of trades. Simply resampling trades doesn't preserve the sequential nature of equity curves.

### The Correct Approach

The correct method:
1. Bootstrap full equity curves from the historical data
2. For each bootstrapped curve, compute the drawdown distribution via nested bootstrap
3. Find the quantiles of these drawdown distributions
4. Use these quantiles as bounds for future drawdown

This approach properly accounts for the path-dependent nature of drawdown and provides more realistic risk estimates.

## References

This implementation is based on research into bootstrap methods for estimating drawdown distributions in trading systems. The methodology demonstrates the importance of proper statistical techniques when dealing with path-dependent risk metrics.

## License

Part of the `statn` statistical analysis toolkit.
