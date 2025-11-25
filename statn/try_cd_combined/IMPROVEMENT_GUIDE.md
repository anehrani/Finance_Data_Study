# Improving CD_MA Model Results

## Current Problem
- All beta coefficients are zero (over-regularization)
- No predictive power found in moving average indicators
- In-sample explained variance: 0.000%

## Recommended Changes (in order of priority)

### 1. **Reduce Regularization** (Try first)
```bash
# Try alpha = 0 (pure Ridge regression, no L1 penalty)
cargo run --bin try_cd_ma -- ../../data/XAGUSD.txt --n-test 100 --alpha 0.0

# Or try smaller alpha values
cargo run --bin try_cd_ma -- ../../data/XAGUSD.txt --n-test 100 --alpha 0.1
```

### 2. **Increase Lookback Periods**
```bash
# Longer moving averages (10-100 periods instead of 6-36)
cargo run --bin try_cd_ma -- ../../data/XAGUSD.txt --n-test 100 --alpha 0.0 --lookback-inc 10 --n-long 10

# Even longer for daily data
cargo run --bin try_cd_ma -- ../../data/XAGUSD.txt --n-test 100 --alpha 0.0 --lookback-inc 20 --n-long 10
```

### 3. **Add More Indicator Variety**
```bash
# Enable RSI indicators
cargo run --bin try_cd_ma -- ../../data/XAGUSD.txt --n-test 100 --alpha 0.0 --enable-rsi --rsi-periods 7,14,21,28
```

### 4. **Increase Training Data**
```bash
# Use more training data (reduce test set)
cargo run --bin try_cd_ma -- ../../data/XAGUSD.txt --n-test 50 --alpha 0.0
```

### 5. **Try Different Data**
- Silver (XAGUSD) may not have strong trend-following characteristics
- Try other markets: EURUSD, GBPUSD, stock indices, etc.

## Quick Test Sequence

```bash
# Step 1: No regularization
cargo run --bin try_cd_ma -- ../../data/XAGUSD.txt --n-test 100 --alpha 0.0

# Step 2: Longer lookbacks + no regularization
cargo run --bin try_cd_ma -- ../../data/XAGUSD.txt --n-test 100 --alpha 0.0 --lookback-inc 10 --n-long 10

# Step 3: Add RSI
cargo run --bin try_cd_ma -- ../../data/XAGUSD.txt --n-test 100 --alpha 0.0 --lookback-inc 10 --n-long 10 --enable-rsi --rsi-periods 14,21,28
```

## What to Look For

**Good results:**
- In-sample explained variance > 1-5%
- Some non-zero beta coefficients
- Positive OOS explained variance in cross-validation
- Consistent OOS returns across different test periods

**Warning signs:**
- All betas still zero → indicators don't work for this market
- Very high in-sample variance (>20%) but poor OOS → overfitting
- Negative OOS explained variance → model worse than baseline
