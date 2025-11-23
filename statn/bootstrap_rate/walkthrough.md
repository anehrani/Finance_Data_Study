# Walkthrough - Convert bootdtrapp_rate to Rust

I have converted the C++ code in `statn/bootdtrapp_rate` to a new Rust package `bootstrap_rate` within the `statn` workspace.

## Changes

1.  **Created Package**: Created `statn/bootstrap_rate` with `Cargo.toml`, `src/lib.rs`, and `src/main.rs`.
2.  **Dependencies**: Added `rand` for random number generation and `stats` (local crate) for statistical functions (`normal_cdf`, `inverse_normal_cdf`).
3.  **Implemented Bootstrap Logic**:
    -   `src/bootstrap.rs`: Implemented `boot_conf_pctile` (Percentile Method) and `boot_conf_bca` (Bias-Corrected and accelerated Method).
    -   These functions take a slice of data, a parameter function (closure), and the number of bootstrap replications.
4.  **Implemented Simulation**:
    -   `src/main.rs`: Implemented the main simulation loop from `BOOT_RATIO.CPP`.
    -   Generates synthetic trade data.
    -   Calculates Profit Factor and Sharpe Ratio.
    -   Computes confidence intervals using Percentile, BCa, and Pivot methods.
    -   Prints statistics comparing the confidence intervals to the true values.
5.  **Documentation**: Added `README.md` with usage instructions.

## Verification

I verified the implementation by running the simulation with sample parameters:

```bash
cargo run -p bootstrap_rate -- 100 100 10 0.6
```

The output confirms that the confidence intervals are calculated and statistics are reported, matching the expected behavior of the original C++ program.

## Files

-   `statn/bootstrap_rate/Cargo.toml`
-   `statn/bootstrap_rate/src/lib.rs`
-   `statn/bootstrap_rate/src/bootstrap.rs`
-   `statn/bootstrap_rate/src/main.rs`
-   `statn/bootstrap_rate/README.md`
