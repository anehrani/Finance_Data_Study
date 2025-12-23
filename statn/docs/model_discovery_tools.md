# statn Model Discovery Tools (try_*)

The `try_*` packages are experimental tools used to discover and optimize trading models using different algorithms and indicator combinations.

## Tools

### [try_cd_ma](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/try_cd_ma)
Focuses on finding optimal Simple Moving Average (SMA) crossover indicators.
- **Algorithm**: Coordinate Descent for Elastic Net regression.
- **Output**: `CD_MA.LOG` containing the selected indicators and their weights.

### [try_cd_comb](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/try_cd_comb)
A more flexible version that combines multiple indicator types (MA, RSI, EMA, MACD, ROC).
- **Algorithm**: Coordinate Descent with Cross-Validation.
- **Features**: Generates a massive grid of indicator candidates and lets the Elastic Net penalty select the most predictive ones.

### [try_diff_ev](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/try_diff_ev)
Uses Differential Evolution to find global optima for model parameters.
- **Algorithm**: Differential Evolution (DE).
- **Use Case**: When the objective function is non-differentiable or has many local optima.

## Common Features
- **Backtesting**: These tools usually include a built-in backtest to evaluate the OOS (Out-of-Sample) performance of the discovered model.
- **Configuration**: Driven by command-line arguments or `config.toml` files.
