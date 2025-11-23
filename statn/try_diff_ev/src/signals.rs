//! Signal generation module for moving average crossover strategy.

/// Result of the signal generation.
#[derive(Debug, Clone)]
pub struct SignalResult {
    /// The raw price series.
    pub prices: Vec<f64>,
    /// Signal per price point: 1 = BUY, -1 = SELL, 0 = HOLD.
    pub signals: Vec<i32>,
    /// Parameters used for the generation (for reference).
    pub long_lookback: usize,
    pub short_pct: f64,
    pub short_thresh: f64,
    pub long_thresh: f64,
}

/// Generate BUY/SELL/HOLD signals for a price slice.
///
/// * `prices` – vector of closing prices (in log space).
/// * `long_lookback` – integer look‑back period for the long moving average.
/// * `short_pct` – percentage (0‑100) of `long_lookback` that defines the short window.
/// * `short_thresh` – threshold (×10 000) for a short‑side sell signal.
/// * `long_thresh` – threshold (×10 000) for a long‑side buy signal.
pub fn generate_signals(
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
        // Calculate the percentage change (ratio - 1), matching test_system logic
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
        short_thresh: short_thresh * 10000.0, // store original format
        long_thresh: long_thresh * 10000.0,   // store original format
    }
}
