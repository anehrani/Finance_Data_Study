# Bug Fix: Index Out of Bounds in compute_targets

## Problem

When running `try_cd_comb`, the program crashed with:
```
thread 'main' panicked at src/core/io/data.rs:63:13:
index out of bounds: the len is 452 but the index is 452
```

## Root Cause

The issue was in the `split_train_test` function in `statn/src/core/io/data.rs`.

### The Problem
When computing target returns, we need to access `prices[i+1] - prices[i]`. This means:
- To compute `n_test` target returns, we need `n_test + 1` prices
- The old code only allocated `max_lookback + n_test` prices for test data
- When computing the last target, it tried to access `prices[max_lookback + n_test]`, which was out of bounds

### Example
If we have:
- `max_lookback = 200`
- `n_test = 252`
- Test data length = 452 (200 + 252)

When computing targets starting at index 200:
- Target 0: `prices[201] - prices[200]` ✓
- Target 1: `prices[202] - prices[201]` ✓
- ...
- Target 251: `prices[452] - prices[451]` ✗ **OUT OF BOUNDS!**

We need index 452, but the array only has indices 0-451.

## Solution

Updated `split_train_test` to allocate **one extra price** for computing the last target:

### Before
```rust
let n_train = data.len() - n_test - max_lookback;
let train_end = max_lookback + n_train;
let test_start = train_end - max_lookback;
```

### After
```rust
// Account for extra price needed for last target
let n_train = data.len() - max_lookback - n_test - 1;
let train_end = max_lookback + n_train + 1;  // +1 for last train target
let test_start = train_end - max_lookback - 1;  // -1 to include extra price
```

Now test data has length `max_lookback + n_test + 1 = 453`, so we can safely access index 452.

## Data Layout

### Old (Broken)
```
Total data: [0 ........................... 8024]
                                            ↑
                                         len=8025

Train: [0 ............... 7572]
        ↑                 ↑
    max_lookback=200   train_end

Test:           [7372 .......... 8024]
                 ↑                ↑
            test_start      len=453 ✗ WRONG!
                            (should be 453)
```

### New (Fixed)
```
Total data: [0 ........................... 8024]
                                            ↑
                                         len=8025

Train: [0 ............... 7573]
        ↑                 ↑
    max_lookback=200   train_end (+1)

Test:           [7372 .......... 8024]
                 ↑                ↑
            test_start      len=453 ✓ CORRECT!
                            (200 + 252 + 1)
```

## Files Modified

1. **`statn/src/core/io/data.rs`**
   - Fixed `split_train_test` function
   - Updated test to verify correct length
   - Added detailed comments explaining the logic

2. **`statn/try_cd_comb/src/data.rs`**
   - Updated test to match new behavior

## Verification

All tests now pass:
```bash
cargo test -p statn --lib core::io::data::tests  # ✓ 5 passed
cargo test -p try_cd_comb                         # ✓ 9 passed
```

The program should now run successfully:
```bash
cargo run -p try_cd_comb -- 10 20 10 0.5 ../../data/XAGUSD.txt --rsi-periods 14,21,28
```

## Impact

This fix ensures that:
1. ✅ Training data has enough prices to compute all training targets
2. ✅ Test data has enough prices to compute all test targets
3. ✅ No index out of bounds errors
4. ✅ Proper overlap between train and test for lookback periods
