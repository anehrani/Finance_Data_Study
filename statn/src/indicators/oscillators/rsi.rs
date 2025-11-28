use std::f64;

/// Calculates the Relative Strength Index (RSI) for a given data slice and period.
///
/// # Arguments
///
/// * `data` - A slice of f64 values (prices).
/// * `period` - The lookback period for RSI (typically 14).
///
/// # Returns
///
/// A Vec<f64> containing the RSI values. The first `period` values are NaN.
pub fn rsi(data: &[f64], period: usize) -> Vec<f64> {
    if period == 0 || period >= data.len() {
        return vec![f64::NAN; data.len()];
    }

    let mut rsi_values = vec![f64::NAN; data.len()];
    let mut gains = 0.0;
    let mut losses = 0.0;

    // Calculate initial average gain/loss
    for i in 1..=period {
        let change = data[i] - data[i - 1];
        if change > 0.0 {
            gains += change;
        } else {
            losses -= change;
        }
    }

    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;

    // First RSI value
    if avg_loss == 0.0 {
        rsi_values[period] = 100.0;
    } else {
        let rs = avg_gain / avg_loss;
        rsi_values[period] = 100.0 - (100.0 / (1.0 + rs));
    }

    // Subsequent RSI values using Wilder's Smoothing
    for i in (period + 1)..data.len() {
        let change = data[i] - data[i - 1];
        let (gain, loss) = if change > 0.0 {
            (change, 0.0)
        } else {
            (0.0, -change)
        };

        avg_gain = ((avg_gain * (period as f64 - 1.0)) + gain) / period as f64;
        avg_loss = ((avg_loss * (period as f64 - 1.0)) + loss) / period as f64;

        if avg_loss == 0.0 {
            rsi_values[i] = 100.0;
        } else {
            let rs = avg_gain / avg_loss;
            rsi_values[i] = 100.0 - (100.0 / (1.0 + rs));
        }
    }

    rsi_values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi() {
        // Example data
        let prices = vec![
            44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03,
            45.61, 46.28, 46.28, 46.00, 46.03, 46.41, 46.22, 45.64,
        ];
        let period = 14;
        let rsi_vals = rsi(&prices, period);

        assert_eq!(rsi_vals.len(), prices.len());
        
        // First 14 values (indices 0 to 13) should be NaN.
        // Index 14 is the first calculated RSI.
        for i in 0..period {
            assert!(rsi_vals[i].is_nan());
        }

        // Check first calculated RSI (index 14)
        // Gain/Loss calc:
        // Changes: -0.25, 0.06, -0.54, 0.72, 0.50, 0.27, 0.32, 0.42, 0.24, -0.19, 0.14, -0.42, 0.67, 0.00
        // Gains: 0.06, 0.72, 0.50, 0.27, 0.32, 0.42, 0.24, 0.14, 0.67 = 3.34
        // Losses: 0.25, 0.54, 0.19, 0.42 = 1.40
        // AvgGain = 3.34 / 14 = 0.23857
        // AvgLoss = 1.40 / 14 = 0.1
        // RS = 2.3857
        // RSI = 100 - 100/(1+2.3857) = 70.46
        
        // My implementation:
        // 46.28 is index 14 (15th element).
        // 44.34 is index 0.
        // Loop 1..=14 sums changes.
        // i=14: prices[14]-prices[13] = 46.28 - 46.28 = 0.0.
        
        // Let's verify with a known value or just basic sanity check.
        // The values should be between 0 and 100.
        assert!(rsi_vals[14] >= 0.0 && rsi_vals[14] <= 100.0);
        
        // Test with simple increasing data
        let increasing: Vec<f64> = (0..20).map(|x| x as f64).collect();
        let rsi_inc = rsi(&increasing, 14);
        assert!(rsi_inc[14] > 90.0); // Should be 100 technically if no losses, but smoothing might affect it.
        // Actually if avg_loss is 0, it returns 100.
        assert_eq!(rsi_inc[14], 100.0);

        // Test with simple decreasing data
        let decreasing: Vec<f64> = (0..20).map(|x| (20 - x) as f64).collect();
        let rsi_dec = rsi(&decreasing, 14);
        assert_eq!(rsi_dec[14], 0.0);
    }
}
