# per_what

A Rust implementation of the `PER_WHAT` C++ program. This tool demonstrates different ways to compute Out-Of-Sample (OOS) returns using a primitive long-only moving-average breakout system with walk-forward optimization.

## Usage

Run the tool using `cargo run`:

```bash
cargo run -p per_what -- [OPTIONS] --filename <FILENAME>
```

### Options

- `--which-crit <INT>`: Optimization criterion (default: 1)
  - `0`: Mean return
  - `1`: Profit factor
  - `2`: Sharpe ratio
- `--all-bars <INT>`: Include all bars in return, even those with no position? (default: 0)
  - `0`: No
  - `1`: Yes
- `--ret-type <INT>`: Return type for testing (default: 2)
  - `0`: All bars
  - `1`: Bars with position open
  - `2`: Completed trades
- `--max-lookback <INT>`: Maximum moving-average lookback (default: 100)
- `--n-train <INT>`: Number of bars in training set (default: 2000)
- `--n-test <INT>`: Number of bars in test set (default: 1000)
- `--filename <FILE>`: Path to market data file (YYYYMMDD Price format)

### Example

```bash
cargo run -p per_what -- \
  --which-crit 1 \
  --all-bars 0 \
  --ret-type 2 \
  --max-lookback 100 \
  --n-train 2000 \
  --n-test 1000 \
  --filename "path/to/market_data.txt"
```

## Build

To build the package:

```bash
cargo build -p per_what
```
