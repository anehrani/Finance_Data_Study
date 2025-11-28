//! Signal generation module for moving average crossover strategy.
//! Contains multiple signal generator implementations.

pub use backtesting::SignalResult;

// SignalResult is now imported from backtesting crate.

/// Dispatch function to select signal generator by name.
///
/// * `generator_type` - Name of the generator ("original" or "log_diff").
/// * ... other args ...
pub fn generate_signals(
    generator_type: &str,
    prices: &[f64],
    long_lookback: usize,
    short_pct: f64,
    short_thresh: f64,
    long_thresh: f64,
) -> SignalResult {
    match generator_type {
        "log_diff" | "enhanced" => generate_signals_log_diff(prices, long_lookback, short_pct, short_thresh, long_thresh),
        "original" => generate_signals_original(prices, long_lookback, short_pct, short_thresh, long_thresh),
        _ => {
            eprintln!("Warning: Unknown generator type '{}', defaulting to 'original'", generator_type);
            generate_signals_original(prices, long_lookback, short_pct, short_thresh, long_thresh)
        }
    }
}

/// Original signal generator (Ratio of log-prices).
///
/// Logic: short_ma / long_ma - 1.0
pub fn generate_signals_original(
    prices: &[f64],
    long_lookback: usize,
    short_pct: f64,
    short_thresh: f64,
    long_thresh: f64,
) -> SignalResult {
    // Compute short window length (rounded to nearest integer).
    let short_lookback = ((short_pct / 100.0) * long_lookback as f64).round() as usize;
    let short_lookback = short_lookback.max(1).min(long_lookback - 1);

    // Convert thresholds from ×10000 format to actual fractions
    let short_thresh = short_thresh / 10000.0;
    let long_thresh = long_thresh / 10000.0;

    // Simple SMA implementation.
    let mut long_ma = vec![0.0; prices.len()];
    let mut short_ma = vec![0.0; prices.len()];

    for i in long_lookback..prices.len() {
        long_ma[i] = prices[i - long_lookback..i].iter().sum::<f64>() / long_lookback as f64;
    }
    for i in short_lookback..prices.len() {
        short_ma[i] = prices[i - short_lookback..i].iter().sum::<f64>() / short_lookback as f64;
    }

    // Build the signal vector.
    let mut signals = vec![0i32; prices.len()];
    for i in 0..prices.len() {
        if i < long_lookback.max(short_lookback) {
            continue; // not enough data yet
        }
        // Original logic: ratio of log-prices
        let change = short_ma[i] / long_ma[i] - 1.0;
        if change > long_thresh {
            signals[i] = 1; // BUY
        } else if change < -short_thresh {
            signals[i] = -1; // SELL
        } else {
            signals[i] = 0; // HOLD
        }
    }

    SignalResult {
        prices: prices.to_vec(),
        signals,
        long_lookback,
        short_pct,
        short_thresh: short_thresh * 10000.0,
        long_thresh: long_thresh * 10000.0,
    }
}

/// Enhanced signal generator (Difference of log-prices).
///
/// Logic: short_ma - long_ma
pub fn generate_signals_log_diff(
    prices: &[f64],
    long_lookback: usize,
    short_pct: f64,
    short_thresh: f64,
    long_thresh: f64,
) -> SignalResult {
    // Compute short window length (rounded to nearest integer).
    let short_lookback = ((short_pct / 100.0) * long_lookback as f64).round() as usize;
    let short_lookback = short_lookback.max(1).min(long_lookback - 1);

    // Convert thresholds from ×10000 format to actual fractions
    let short_thresh = short_thresh / 10000.0;
    let long_thresh = long_thresh / 10000.0;

    // Simple SMA implementation.
    let mut long_ma = vec![0.0; prices.len()];
    let mut short_ma = vec![0.0; prices.len()];

    for i in long_lookback..prices.len() {
        long_ma[i] = prices[i - long_lookback..i].iter().sum::<f64>() / long_lookback as f64;
    }
    for i in short_lookback..prices.len() {
        short_ma[i] = prices[i - short_lookback..i].iter().sum::<f64>() / short_lookback as f64;
    }

    // Build the signal vector.
    let mut signals = vec![0i32; prices.len()];
    for i in 0..prices.len() {
        if i < long_lookback.max(short_lookback) {
            continue; // not enough data yet
        }
        // Correct logic: difference of log-prices
        let change = short_ma[i] - long_ma[i];
        
        if change > long_thresh {
            signals[i] = 1; // BUY
        } else if change < -short_thresh {
            signals[i] = -1; // SELL
        } else {
            signals[i] = 0; // HOLD
        }
    }

    SignalResult {
        prices: prices.to_vec(),
        signals,
        long_lookback,
        short_pct,
        short_thresh: short_thresh * 10000.0,
        long_thresh: long_thresh * 10000.0,
    }
}
