# statn Core Module

The `core` module provides the fundamental building blocks for the `statn` library, including data structures, I/O utilities, mathematical libraries, and statistical functions.

## Submodules

### [IO](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/core/io)
Handles reading and writing market data and general data structures.
- **market.rs**: Contains `OhlcData` struct and functions to read price/OHLC files (`read_price_file`, `read_ohlc_file`).
- **read.rs** / **write.rs**: General utilities for data I/O.
- **data.rs**: Core data structures used throughout the library.

### [Matlib](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/core/matlib)
A custom matrix and math library.
- **linalg.rs**: Basic linear algebra operations.
- **rands.rs** / **mwc256.rs**: Random number generators (including MWC256).
- **qsorts.rs**: Efficient sorting algorithms.
- **paramcor.rs**: Tools for parameter correlation and matrix manipulation.

### [Stats](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/src/core/stats)
A comprehensive collection of statistical functions and tests.
- **Distribution Functions**: Normal CDF, Inverse Normal, Gamma, Beta, Student's t, F-distribution, Poisson, etc.
- **Statistical Tests**:
    - T-tests (one-sample, two-sample).
    - Non-parametric tests: Mann-Whitney U, Kolmogorov-Smirnov, Anderson-Darling.
    - ANOVA and Kruskal-Wallis.
    - Chi-square and association measures (Nominal Lambda, Uncertainty Reduction).

## Key Structures

### `OhlcData`
```rust
pub struct OhlcData {
    pub date: Vec<u32>,
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
}
```
Used for storing and manipulating time-series market data.
