use crate::trend::ma::moving_average;

/// Computes the Commodity Channel Index (CCI).
///
/// # Arguments
///
/// * `highs` - A slice of high prices.
/// * `lows` - A slice of low prices.
/// * `closes` - A slice of closing prices.
/// * `period` - The lookback period (typically 20).
///
/// # Returns
///
/// A Vec<f64> containing the CCI values.
pub fn cci(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
) -> Vec<f64> {
    let n = closes.len();
    if n != highs.len() || n != lows.len() || period == 0 || period > n {
        return vec![f64::NAN; n];
    }

    // 1. Typical Price (TP)
    let mut tp = Vec::with_capacity(n);
    for i in 0..n {
        tp.push((highs[i] + lows[i] + closes[i]) / 3.0);
    }

    // 2. SMA(TP)
    let sma_tp = moving_average(&tp, period);

    let mut cci_values = Vec::with_capacity(n);

    for i in 0..n {
        if i < period - 1 {
            cci_values.push(f64::NAN);
            continue;
        }

        let current_tp = tp[i];
        let current_sma = sma_tp[i];

        // 3. Mean Deviation
        let slice_tp = &tp[i + 1 - period..=i];
        let mean_dev: f64 = slice_tp.iter()
            .map(|&val| (val - current_sma).abs())
            .sum::<f64>() / period as f64;

        // 4. CCI
        if mean_dev.abs() < 1e-10 {
            cci_values.push(0.0);
        } else {
            cci_values.push((current_tp - current_sma) / (0.015 * mean_dev));
        }
    }

    cci_values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cci() {
        // Constant prices -> CCI should be 0
        let highs = vec![10.0; 10];
        let lows = vec![10.0; 10];
        let closes = vec![10.0; 10];
        let period = 5;

        let output = cci(&highs, &lows, &closes, period);

        assert_eq!(output.len(), 10);
        assert!(output[3].is_nan());
        assert_eq!(output[4], 0.0);
    }

    #[test]
    fn test_cci_trend() {
        // Linearly increasing prices
        // TP: 10, 11, 12, 13, 14
        // SMA(5): 12
        // Mean Dev: (|10-12| + |11-12| + |12-12| + |13-12| + |14-12|) / 5
        //         = (2 + 1 + 0 + 1 + 2) / 5 = 6/5 = 1.2
        // CCI = (14 - 12) / (0.015 * 1.2) = 2 / 0.018 = 111.111...

        let highs = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let lows = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let closes = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let period = 5;

        let output = cci(&highs, &lows, &closes, period);

        assert!((output[4] - 111.111111).abs() < 1e-4);
    }
}
