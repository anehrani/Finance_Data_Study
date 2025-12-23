# statn Backtesting Module

The `backtesting` module provides a lightweight engine for simulating trading strategies based on generated signals.

## Structure

### [Core](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/backtesting/src/core.rs)
- **`backtest_signals`**: The primary entry point for running a backtest. It takes a series of signals and price data, simulating entry/exit logic and calculating trade-by-trade performance.

### [Models](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/backtesting/src/models.rs)
- **`SignalResult`**: Represents the outcome of a signal evaluation at a specific time step.
- **`TradeLog`**: Records individual trade details (entry time, exit time, return, etc.).
- **`TradeStats`**: Aggregates performance metrics (e.g., total return, Sharpe ratio, max drawdown).

## Usage
The backtesting engine is typically used after a model has generated predictive signals. It converts these signals into hypothetical equity curves and performance statistics.
