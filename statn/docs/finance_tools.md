# statn Finance Tools Module

The `finance_tools` module contains utility functions specifically tailored for financial data analysis and risk management.

## Functions

### [Price](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/finance_tools/price.rs)
- **`atr(lookback, high, low, close, index)`**: Calculates the Average True Range (ATR) over a specified lookback period at a given index. ATR is a common measure of volatility.

### [Probability](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/finance_tools/probability.rs)
- **`clean_tails(raw, tail_frac)`**: A robust outlier management routine. It "cleans" the tails of a distribution by applying an exponential decay to values outside the central `(1.0 - 2.0 * tail_frac)` range. This is useful for tempering the influence of extreme events in training data.
