# MACD Indicator Addition Summary

## What Was Added

### 1. MACD Indicator Implementation
**Location**: `statn/src/indicators/oscillators/macd.rs`

Implemented a complete MACD (Moving Average Convergence Divergence) indicator with:
- **EMA calculation**: Exponential Moving Average with configurable period
- **MACD line**: Fast EMA - Slow EMA
- **Signal line**: EMA of MACD line
- **Histogram**: MACD line - Signal line (primary trading signal)
- **Configurable parameters**: Fast (default 12), Slow (default 26), Signal (default 9)
- **Comprehensive tests**: 5 test cases covering EMA, MACD, histogram, crossovers

### 2. Integration with try_cd_comb

**Configuration** (`config.rs`):
- Added `include_macd: bool` field to `Config` struct
- Added `--include-macd` command-line flag
- Updated `n_vars()` to include MACD count
- Updated `max_lookback()` to account for MACD's 35-period requirement

**Indicator Specification** (`indicators.rs`):
- Added `Macd` variant to `IndicatorSpec` enum
- Updated `generate_specs()` to include MACD when enabled
- Updated `compute_all_indicators()` to compute MACD histogram
- Uses standard MACD parameters (12, 26, 9)

**Evaluation** (`evaluation.rs`):
- Added MACD coefficient display section
- Shows whether MACD was selected (non-zero coefficient)

**Configuration Files**:
- Updated `config.example.toml` with `include_macd = true`
- Added documentation for MACD parameters

## How MACD Works

### Technical Details

**MACD** is a trend-following momentum indicator that shows the relationship between two moving averages:

1. **Fast EMA (12)**: Short-term exponential moving average
2. **Slow EMA (26)**: Long-term exponential moving average  
3. **MACD Line**: Fast EMA - Slow EMA
4. **Signal Line**: 9-period EMA of MACD line
5. **Histogram**: MACD line - Signal line

### Trading Signals

- **Positive histogram**: Bullish momentum (MACD above signal)
- **Negative histogram**: Bearish momentum (MACD below signal)
- **Histogram increasing**: Strengthening trend
- **Histogram decreasing**: Weakening trend
- **Zero crossover**: Potential trend change

### In the Combined System

The MACD histogram is used as a **single indicator** in the elastic net model:
- If selected (non-zero coefficient), it contributes to the combined prediction
- Positive coefficient: Align with MACD signal (buy when histogram positive)
- Negative coefficient: Inverse MACD signal (contrarian strategy)

## Usage Examples

### With Configuration File
```toml
# config.toml
include_macd = true
rsi_periods = [14, 21, 28]
```

```bash
cargo run -p try_cd_comb -- --config config.toml
```

### With Command Line
```bash
# All indicators: MA + RSI + MACD
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt --rsi-periods 14,21,28 --include-macd

# Just MA + MACD (no RSI)
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt --include-macd

# Just MA (no RSI, no MACD)
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt
```

## Example Output

```
Configuration:
  Number of indicators: 204  # 200 MA + 3 RSI + 1 MACD

Beta Coefficients:
Row: long-term lookback | Columns: short-term lookback
   10    0.1234    0.0000    ----    ...
   20    ----      0.2345    ----    ...

RSI Coefficients:
  Period  14:    0.0567
  Period  21:      ----
  Period  28:   -0.0234

MACD Coefficient (Histogram):
  MACD:    0.0123

Out-of-Sample Results:
  Total return: 0.12345 (13.145%)
```

## Testing

All tests pass:
```bash
# Test MACD indicator
cargo test -p indicators oscillators::macd  # ✓ 5 passed

# Test full integration
cargo test -p try_cd_comb                   # ✓ 9 passed
```

## Technical Implementation

### MACD Calculation Details

```rust
// 1. Calculate Fast and Slow EMAs
let fast_ema = ema(prices, 12);
let slow_ema = ema(prices, 26);

// 2. MACD Line = Fast - Slow
let macd_line = fast_ema - slow_ema;

// 3. Signal Line = EMA of MACD
let signal_line = ema(macd_line, 9);

// 4. Histogram = MACD - Signal (used as indicator)
let histogram = macd_line - signal_line;
```

### EMA Formula

```
EMA[t] = (Price[t] - EMA[t-1]) × Multiplier + EMA[t-1]

where Multiplier = 2 / (Period + 1)
```

### Lookback Requirement

MACD needs **35 periods** minimum:
- 26 for slow EMA
- 9 for signal line EMA
- Total: 26 + 9 = 35

## Files Modified/Created

### Created:
- `statn/src/indicators/oscillators/macd.rs` (new MACD implementation)

### Modified:
- `statn/src/indicators/oscillators/mod.rs` (expose MACD module)
- `statn/try_cd_comb/src/config.rs` (add include_macd field)
- `statn/try_cd_comb/src/indicators.rs` (add MACD to IndicatorSpec)
- `statn/try_cd_comb/src/evaluation.rs` (display MACD coefficient)
- `statn/try_cd_comb/main.rs` (pass include_macd parameter)
- `statn/try_cd_comb/config.example.toml` (add MACD option)
- `statn/try_cd_comb/README.md` (document MACD support)

## Benefits

1. **Trend + Momentum**: MACD captures both trend direction and momentum strength
2. **Complementary to MA**: While MA crossovers show trend, MACD shows momentum
3. **Widely Used**: One of the most popular technical indicators
4. **Automatic Selection**: Elastic net decides if MACD improves predictions
5. **Standard Parameters**: Uses well-established 12, 26, 9 configuration

## Future Enhancements

Potential improvements:
- **Custom MACD parameters**: Allow user to specify fast/slow/signal periods
- **MACD line and signal**: Use both separately instead of just histogram
- **Multiple MACD configs**: Test different parameter combinations
- **MACD divergence**: Detect divergence between price and MACD

## Comparison with Other Indicators

| Indicator | Type | Lookback | Signal |
|-----------|------|----------|--------|
| MA Crossover | Trend | Variable (10-200) | Short MA - Long MA |
| RSI | Momentum | Variable (14-28) | 0-100 scale |
| MACD | Trend + Momentum | 35 | Histogram |

All three complement each other:
- **MA**: Pure trend following
- **RSI**: Overbought/oversold conditions
- **MACD**: Trend momentum and strength
