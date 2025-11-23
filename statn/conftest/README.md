# conftest

`conftest` is a Rust application converted from C++ that tests quantile confidence intervals via incomplete beta function.

## Usage

```bash
cargo run -p conftest -- [nsamples] [fail_rate] [low_q] [high_q] [p_of_q]
```

If no arguments are provided, it runs with default values:
- nsamples: 100000
- fail_rate: 0.1
- low_q: 0.0975
- high_q: 0.101
- p_of_q: 0.01

## Description

The program runs an infinite simulation loop (stop with Ctrl-C).
In each iteration, it:
1. Generates `nsamples` random numbers from a uniform distribution.
2. Sorts them.
3. Checks if the lower and upper bounds (defined by `fail_rate`) fall within expected quantiles.
4. Accumulates and prints statistics about failure rates.

This is used to verify the correctness of confidence interval calculations.
