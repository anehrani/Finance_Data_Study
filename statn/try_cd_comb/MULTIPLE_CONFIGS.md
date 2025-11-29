# Multiple MACD and RSI Configurations - Final Implementation

## Summary

Successfully enhanced `try_cd_comb` to support **multiple configurations** of both MACD and RSI indicators, similar to how the system handles multiple MA crossover combinations. This allows the elastic net to optimize across different parameter settings for each indicator type.

## What Changed

### 1. Configuration System

**Before:**
- Single `include_macd: bool` flag
- Fixed RSI periods via `rsi_periods: Vec<usize>`

**After:**
- `macd_configs: Vec<(usize, usize, usize)>` - Multiple MACD parameter sets
- Each tuple is `(fast_period, slow_period, signal_period)`
- RSI periods unchanged (already supported multiple configs)

### 2. Indicator Specification

**Updated `IndicatorSpec` enum:**
```rust
pub enum IndicatorSpec {
    MaCrossover { short_lookback: usize, long_lookback: usize },
    Rsi { period: usize },
    Macd { fast: usize, slow: usize, signal: usize },  // ← Now includes parameters
}
```

### 3. Example Configuration

**config.toml:**
```toml
# Multiple RSI periods
rsi_periods = [14, 21, 28]

# Multiple MACD configurations
macd_configs = [
    [12, 26, 9],   # Standard MACD
    [5, 35, 5],    # Fast MACD  
    [19, 39, 9],   # Slow MACD
]
```

This creates:
- **200 MA crossovers** (20 long × 10 short)
- **3 RSI indicators** (periods 14, 21, 28)
- **3 MACD indicators** (standard, fast, slow)
- **Total: 206 indicators**

## Usage Examples

### With Configuration File

```toml
# config.toml
lookback_inc = 10
n_long = 20
n_short = 10
alpha = 0.5
data_file = "data/XAGUSD.txt"

rsi_periods = [14, 21, 28]
macd_configs = [
    [12, 26, 9],   # Standard
    [5, 35, 5],    # Fast
    [19, 39, 9],   # Slow
]
```

```bash
cargo run -p try_cd_comb --release -- --config config.toml
```

### With Command Line

```bash
# With default MACD (12,26,9) only
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt --rsi-periods 14,21,28 --include-macd

# Without any MACD (just MA + RSI)
cargo run -p try_cd_comb -- 10 20 10 0.5 data/XAGUSD.txt --rsi-periods 14,21,28
```

## Output Example

```
Configuration:
  Number of indicators: 206  # 200 MA + 3 RSI + 3 MACD

Beta Coefficients (In-sample explained variance: 45.2%):
Row: long-term lookback | Columns: short-term lookback
   10    0.1234    0.0000    ----    ...
   20    ----      0.2345    ----    ...

RSI Coefficients:
  Period  14:    0.0567
  Period  21:      ----
  Period  28:   -0.0234

MACD Coefficients (Histogram):
  MACD(12,26,9):    0.0123
  MACD(5,35,5):       ----
  MACD(19,39,9):   -0.0089

Out-of-Sample Results:
  Total return: 0.12345 (13.145%)
```

## MACD Parameter Variations

### Standard MACD (12, 26, 9)
- **Fast EMA**: 12 periods
- **Slow EMA**: 26 periods
- **Signal**: 9 periods
- **Use case**: General purpose, most widely used
- **Lookback needed**: 35 periods (26 + 9)

### Fast MACD (5, 35, 5)
- **Fast EMA**: 5 periods
- **Slow EMA**: 35 periods
- **Signal**: 5 periods
- **Use case**: More responsive to recent price changes
- **Lookback needed**: 40 periods (35 + 5)

### Slow MACD (19, 39, 9)
- **Fast EMA**: 19 periods
- **Slow EMA**: 39 periods
- **Signal**: 9 periods
- **Use case**: Smoother, less sensitive to noise
- **Lookback needed**: 48 periods (39 + 9)

## How Elastic Net Selects Indicators

The system now has **206 candidate indicators**:

1. **MA Crossovers (200)**: Different trend timeframes
2. **RSI (3)**: Different momentum periods
3. **MACD (3)**: Different parameter sensitivities

**Elastic Net automatically:**
- Selects the most predictive indicators
- Sets coefficients to zero for redundant/useless indicators
- Handles multicollinearity (e.g., if standard and fast MACD are correlated)
- Creates a sparse, interpretable model

**Example selection:**
```
Selected indicators (non-zero coefficients):
- MA(10/5): 0.1234      ← Short-term trend
- MA(200/100): 0.0567   ← Long-term trend
- RSI(14): 0.0234       ← Standard momentum
- MACD(12,26,9): 0.0123 ← Standard MACD
- MACD(19,39,9): -0.0089 ← Slow MACD (inverse signal)

Not selected (zero coefficients):
- 195 other MA crossovers
- RSI(21), RSI(28)
- MACD(5,35,5)
```

## Benefits of Multiple Configurations

### 1. **Automatic Parameter Optimization**
- No need to manually test different MACD/RSI parameters
- Elastic net finds the best combination
- Cross-validation prevents overfitting

### 2. **Complementary Signals**
- Fast MACD: Catches quick reversals
- Slow MACD: Confirms longer-term trends
- Different RSI periods: Various momentum timeframes

### 3. **Robustness**
- If one parameter set fails, others may work
- Diversification across indicator configurations
- Reduces dependency on single parameter choice

### 4. **Interpretability**
- See which parameter sets are selected
- Understand which timeframes matter
- Positive vs negative coefficients show strategy direction

## Technical Implementation

### Lookback Calculation

The system automatically calculates the maximum lookback needed:

```rust
pub fn max_lookback(&self) -> usize {
    let ma_max = self.n_long * self.lookback_inc;
    let rsi_max = self.rsi_periods.iter().max().unwrap_or(0);
    let macd_max = self.macd_configs.iter()
        .map(|(_, slow, signal)| slow + signal)
        .max()
        .unwrap_or(0);
    ma_max.max(rsi_max).max(macd_max)
}
```

**Example:**
- MA: 20 × 10 = 200
- RSI: max(14, 21, 28) = 28
- MACD: max(35, 40, 48) = 48
- **Total lookback: 200**

### Indicator Count

```rust
pub fn n_vars(&self) -> usize {
    let ma_count = self.n_long * self.n_short;      // 200
    let rsi_count = self.rsi_periods.len();         // 3
    let macd_count = self.macd_configs.len();       // 3
    ma_count + rsi_count + macd_count               // 206
}
```

## Testing

All tests pass:
```bash
cargo test -p try_cd_comb  # ✓ 9/9 passed
```

**Test coverage:**
- Multiple MACD configurations
- Parameter validation
- Indicator generation
- Coefficient display
- Edge cases (empty configs)

## Files Modified

1. **`src/config.rs`**
   - Changed `include_macd: bool` → `macd_configs: Vec<(usize, usize, usize)>`
   - Updated `n_vars()` and `max_lookback()`
   - Updated tests

2. **`src/indicators.rs`**
   - Updated `IndicatorSpec::Macd` to include parameters
   - Modified `generate_specs()` to loop through configs
   - Updated `compute_all_indicators()` to use custom configs
   - Updated tests

3. **`src/evaluation.rs`**
   - Loop through MACD configs for coefficient display
   - Show parameters for each MACD

4. **`main.rs`**
   - Pass `macd_configs` instead of `include_macd`

5. **`config.example.toml`**
   - Added example with 3 MACD configurations

## Comparison: Before vs After

| Aspect | Before | After |
|--------|--------|-------|
| MACD configs | 1 (fixed 12,26,9) | Multiple (user-defined) |
| RSI configs | Multiple ✓ | Multiple ✓ |
| MA configs | Multiple ✓ | Multiple ✓ |
| Total indicators | 204 (200 MA + 3 RSI + 1 MACD) | 206+ (200 MA + 3 RSI + 3+ MACD) |
| Flexibility | Limited | High |
| Parameter optimization | Manual | Automatic via elastic net |

## Recommended MACD Configurations

### Conservative (3 configs)
```toml
macd_configs = [
    [12, 26, 9],   # Standard
    [5, 35, 5],    # Fast
    [19, 39, 9],   # Slow
]
```

### Aggressive (5 configs)
```toml
macd_configs = [
    [8, 17, 9],    # Very fast
    [12, 26, 9],   # Standard
    [16, 30, 9],   # Medium
    [19, 39, 9],   # Slow
    [24, 52, 9],   # Very slow
]
```

### Minimal (1 config)
```toml
macd_configs = [
    [12, 26, 9],   # Standard only
]
```

Or use command line:
```bash
--include-macd  # Adds standard MACD (12,26,9)
```

## Performance Considerations

- **More indicators = More computation**: Each MACD requires EMA calculations
- **More indicators = More memory**: Storing all indicator values
- **Elastic net handles it**: Regularization prevents overfitting even with many indicators
- **Cross-validation**: Ensures selected indicators generalize to test data

**Recommendation**: Start with 3-5 MACD configs, monitor performance

## Future Enhancements

Potential additions:
- **MACD line and signal separately**: Currently only uses histogram
- **MACD divergence detection**: Price vs MACD divergence signals
- **Adaptive MACD**: Parameters that change based on volatility
- **MACD slope**: Rate of change of histogram

## Conclusion

The system now provides **complete flexibility** for indicator parameter optimization:

✅ **Multiple MA crossovers** - Different trend timeframes  
✅ **Multiple RSI periods** - Different momentum windows  
✅ **Multiple MACD configs** - Different sensitivity levels  

All optimized automatically via **elastic net regularization** with **cross-validation** to prevent overfitting!
