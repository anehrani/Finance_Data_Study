# statn Complete Model Generator

`complete_model_generator` is an end-to-end orchestration tool designed to automate the entire workflow of building, testing, and documenting a trading model.

## Workflow Orchestration
The tool executes the following steps in sequence:
1. **Data Conversion**: Converts OHLC data to a simplified price-only format for modeling.
2. **Stationarity Testing**: Runs the `stationary_test` to check for unit roots.
3. **Entropy Analysis**: Runs `check_entropy` to measure the information content of the data.
4. **Model Generation**: Executes `try_cd_ma` to find optimal Moving Average crossover indicators using Coordinate Descent.
5. **Permutation Testing**: Runs `montecarlo_permutation_test` (MCPT) to check for "p-hacking" or data mining bias.
6. **Sensitivity Analysis**: Evaluates how stable the model parameters are under perturbations.
7. **Risk Assessment**: Runs the `drawdown` tool to estimate potential losses.
8. **Cross-Validation**: Performs market-specific cross-validation.
9. **Confidence Testing**: Runs `conftest` to assess the statistical significance of results.
10. **Report Generation**: Consolidates all outputs into a comprehensive `REPORT.md`.

## Usage
```bash
cargo run --release -p complete_model_generator -- <DATA_FILE> [--output-dir <DIR>]
```
- `<DATA_FILE>`: Path to market history.
- `--output-dir`: Where to save logs and the final report (default: `model_report`).
