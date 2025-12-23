# STATN - Statistical Tests of Indicators

`statn` is a comprehensive Rust-based toolkit for quantitative traders and researchers. It provides a systematic pipeline for designing, optimizing, and validating trading models based on statistical principles.

## Core Content
The workspace is organized into a modular library and a suite of specialized application tools:

### Libraries (`statn/src/`)
- [**Core**](statn/docs/core.md): Fundamental utilities for I/O, linear algebra, and a massive library of statistical tests.
- [**Indicators**](statn/docs/indicators.md): Optimized computation of technical indicators (Moving Averages, RSI, MACD, etc.).
- [**Models**](statn/docs/models.md): High-performance solvers like Coordinate Descent for Elastic Net regression and Differential Evolution.
- [**Estimators**](statn/docs/estimators.md): Advanced tools for measuring selection bias and strategy robustness.
- [**Finance Tools**](statn/docs/finance_tools.md) / [**Backtesting**](statn/docs/backtesting.md): Domain-specific math and strategy simulation.

### Application Tools
- [**Complete Model Generator**](statn/docs/complete_model_generator.md): An automated orchestrator that runs the entire pipeline from raw data to a validated model report.
- [**Discovery Tools**](statn/docs/model_discovery_tools.md): Experimental binaries (`try_cd_ma`, `try_cd_comb`) for exploring indicator parameter spaces.
- [**Statistical Utilities**](statn/docs/statistical_tools.md): Standalone tools for entropy checks, stationarity tests, and Monte Carlo Permutation Tests.

## Application Summary
The primary application of `statn` is to eliminate "guessing" in trading strategy design by replacing intuition with a rigorous, math-driven pipeline:

1.  **Ensuring Stationarity**: Validating that indicators don't drift and remain mathematically stable over time.
2.  **Information Content (Entropy)**: Using entropy checks to ensure indicators provide real signal rather than clumped noise.
3.  **Automatic Selection (Regularization)**: Using Coordinate Descent and Elastic Net penalties to automatically filter out hundreds of "garbage" indicator variations, keeping only those with true predictive power.
4.  **Robustness Analysis**: Stress-testing parameters via sensitivity analysis and Permutation Tests to ensure performance isn't just a result of market noise (p-hacking).

For a deep dive into the methodology and detailed tool documentation, see the [**Documentation Landing Page**](statn/docs/README.md).
