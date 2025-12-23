# statn Estimators Module

The `estimators` module provides tools for statistical estimation, optimization, and validation of trading models.

## Tools

### [Stochastic Bias Estimator](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/estimators/stochastic_bias.rs)
Implemented in the `StocBias` struct. It addresses the "selection bias" that occurs when choosing the best performing model from a large set of stochastic trials.
- Estimates the expected "optimization bias" by tracking In-Sample (IS) bests and corresponding Out-of-Sample (OOS) results.

### [Optimization Utilities](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/estimators/glob_max.rs)
- **`brentmax`**: Implementation of Brent's method for finding the maximum of a 1D function.
- **`glob_max`**: Algorithms for global maximization.

### [Sensitivity Analysis](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/estimators/sensitivity.rs)
Tools for measuring how changes in input parameters or data perturbations affect model performance. Useful for assessing model robustness.
