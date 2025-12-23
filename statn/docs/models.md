# statn Models Module

The `models` module contains the implementation of core predictive and optimization models.

## Predictive Models

### [Coordinate Descent (CD)](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/models/cd_ma.rs)
A high-performance implementation of Elastic Net regularized regression using coordinate descent.
- **`CoordinateDescent` struct**: Holds model state, coefficients (`beta`), and standardization parameters.
- **Training Modes**:
    - `core_train`: Single lambda optimization.
    - `lambda_train`: Explores a path of lambda values (shrinkage parameters).
    - `cv_train`: Performs cross-validation to select the optimal lambda.
- **Features**: Supports weighted cases and covariance-based updates for efficiency.

## Optimization Models

### [Differential Evolution](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/models/differential_evolution.rs)
Implements the Differential Evolution (DE) algorithm for global optimization of complex, non-linear objective functions (e.g., finding optimal indicator parameters).
- Supports various DE strategies and population management.
