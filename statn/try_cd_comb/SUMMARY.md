# Summary: Enhanced try_cd_comb with RSI Indicator

## What Was Done

### 1. Created RSI Indicator Module
**Location**: `statn/src/indicators/oscillators/rsi.rs`

- Implemented **Relative Strength Index (RSI)** calculation
- Uses Wilder's Smoothing method (standard RSI calculation)
- Handles edge cases (zero losses → RSI = 100)
- Includes comprehensive tests
- Returns values on 0-100 scale

### 2. Updated Indicators Library Structure
**Location**: `statn/src/indicators/`

- Created new `oscillators` module
- Exposed RSI through `oscillators::rsi`
- Updated `lib.rs` to include oscillators module
- Maintained existing `trend` and `volatility` modules

### 3. Enhanced try_cd_comb Configuration
**Location**: `statn/try_cd_comb/src/config.rs`

**Added**:
- `rsi_periods: Vec<usize>` field to `Config` struct
- `--rsi-periods` command-line argument (comma-separated)
- Updated `n_vars()` to include RSI indicators: `(n_long * n_short) + rsi_periods.len()`
- Updated `max_lookback()` to consider RSI periods

### 4. Updated Indicator Specification System
**Location**: `statn/try_cd_comb/src/indicators.rs`

**Changed**:
- `IndicatorSpec` from struct to enum:
  ```rust
  pub enum IndicatorSpec {
      MaCrossover { short_lookback: usize, long_lookback: usize },
      Rsi { period: usize },
  }
  ```
- `generate_specs()` now creates both MA and RSI specs
- `compute_all_indicators()` handles both indicator types
- Updated tests to verify mixed indicator generation

### 5. Enhanced Evaluation Output
**Location**: `statn/try_cd_comb/src/evaluation.rs`

**Added**:
- RSI coefficient reporting section
- Displays RSI periods and their learned weights
- Maintains existing MA crossover grid display

### 6. Package Renaming
**Location**: `statn/try_cd_comb/`

- Renamed package from `try_cd_ma` to `try_cd_comb`
- Updated `Cargo.toml` package name
- Updated workspace members in root `Cargo.toml`
- Updated import statements in `main.rs`

### 7. Documentation
**Created**:
- `README.md`: Comprehensive documentation
- `config.example.toml`: Sample configuration with RSI

## How to Use

### With Configuration File
```bash
cargo run -p try_cd_comb -- --config config.example.toml
```

### With Command-Line Arguments
```bash
# With RSI
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt --rsi-periods 14,21,28

# Without RSI (MA only)
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt
```

## Example Output

The system will now report:

```
Configuration:
  Number of indicators: 203  # (20 * 10) MA + 3 RSI

Beta Coefficients:
Row: long-term lookback | Columns: short-term lookback
   10    0.1234    0.0000    ----    ...
   20    ----      0.2345    ----    ...

RSI Coefficients:
  Period  14:    0.0567
  Period  21:      ----
  Period  28:   -0.0234
```

## Testing

All tests pass:
```bash
cargo test -p indicators     # RSI tests
cargo test -p try_cd_comb    # Integration tests
```

## Architecture Benefits

1. **Extensible**: Easy to add more indicator types (MACD, Bollinger Bands, etc.)
2. **Type-safe**: Enum-based indicator specs prevent errors
3. **Modular**: Indicators are in separate library, reusable across packages
4. **Configurable**: RSI periods can be specified via config or CLI
5. **Sparse**: Elastic net will select only useful indicators

## Next Steps (Potential Enhancements)

1. **Add MACD** (Moving Average Convergence Divergence)
2. **Add Bollinger Bands** (volatility-based)
3. **Add Stochastic Oscillator** (momentum)
4. **Add ATR** (Average True Range - volatility)
5. **Volume indicators** (if volume data available)
6. **Multiple timeframes** (e.g., daily + weekly indicators)

## Files Modified/Created

### Created:
- `statn/src/indicators/oscillators/rsi.rs`
- `statn/src/indicators/oscillators/mod.rs`
- `statn/try_cd_comb/README.md`
- `statn/try_cd_comb/SUMMARY.md` (this file)

### Modified:
- `statn/src/indicators/lib.rs`
- `statn/try_cd_comb/Cargo.toml`
- `statn/try_cd_comb/main.rs`
- `statn/try_cd_comb/src/config.rs`
- `statn/try_cd_comb/src/indicators.rs`
- `statn/try_cd_comb/src/evaluation.rs`
- `statn/Cargo.toml`

### Updated:
- `statn/try_cd_comb/config.example.toml`
