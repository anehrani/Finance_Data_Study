# statn Indicators Module

The `indicators` module is responsible for defining, generating, and computing technical indicators used in trading models.

## Structure

### [Specs](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/indicators/specs.rs)
Defines how indicators are specified and computed.
- **`IndicatorSpec`**: Enum representing different types of indicators (currently focused on Crossovers).
- **`CrossoverType`**: Supported crossover types: `Ma`, `Rsi`, `Ema`, `Macd`, `Roc`.
- **`IndicatorData`**: Container for computed indicator values and target returns.

### [Oscillators](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/indicators/oscillators)
Implementation of common oscillators.
- **rsi.rs**: Relative Strength Index.
- **macd.rs**: Moving Average Convergence Divergence.

### [Trend](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/indicators/trend)
Implementation of trend-following indicators.
- **ma.rs**: Simple and Exponential Moving Averages.

### [Volatility](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/indicators/volatility)
Implementation of volatility-based indicators.

## Key Functions

### `generate_specs`
Generates a grid of indicator specifications based on lookback increments and counts. This is used for feature discovery.

### `compute_all_indicators`
Computes the values for a given set of `IndicatorSpec` items over a price series.

### `compute_indicator_data`
A high-level function that computes both indicators and the target returns (usually log-price differences) for training/testing.
