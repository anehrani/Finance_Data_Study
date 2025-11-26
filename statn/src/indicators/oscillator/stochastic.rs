use crate::trend::ma::moving_average;

/// Stochastic Oscillator Output
#[derive(Debug, Clone)]
pub struct StochasticOutput {
    /// %K Line
    pub k_line: Vec<f64>,
    /// %D Line (Signal Line)
    pub d_line: Vec<f64>,
}

/// Computes the Stochastic Oscillator.
///
/// # Arguments
///
/// * `highs` - A slice of high prices.
/// * `lows` - A slice of low prices.
/// * `closes` - A slice of closing prices.
/// * `k_period` - The lookback period for %K (typically 14).
/// * `d_period` - The smoothing period for %D (typically 3).
///
/// # Returns
///
/// A `StochasticOutput` struct containing the %K and %D lines.
pub fn stochastic_oscillator(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    k_period: usize,
    d_period: usize,
) -> StochasticOutput {
    let n = closes.len();
    if n != highs.len() || n != lows.len() || k_period == 0 || k_period > n {
        return StochasticOutput {
            k_line: vec![f64::NAN; n],
            d_line: vec![f64::NAN; n],
        };
    }

    let mut k_line = Vec::with_capacity(n);

    for i in 0..n {
        if i < k_period - 1 {
            k_line.push(f64::NAN);
            continue;
        }

        let slice_highs = &highs[i + 1 - k_period..=i];
        let slice_lows = &lows[i + 1 - k_period..=i];

        let highest_high = slice_highs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lowest_low = slice_lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        let current_close = closes[i];
        
        let k = if (highest_high - lowest_low).abs() < 1e-10 {
            50.0 // Avoid division by zero, neutral value
        } else {
            (current_close - lowest_low) / (highest_high - lowest_low) * 100.0
        };

        k_line.push(k);
    }

    // Calculate %D (SMA of %K)
    // We need to handle the initial NaNs in k_line.
    let valid_start_idx = k_period - 1;
    if valid_start_idx >= k_line.len() {
         return StochasticOutput {
            k_line,
            d_line: vec![f64::NAN; n],
        };
    }

    let valid_k_slice = &k_line[valid_start_idx..];
    let valid_d_line = moving_average(valid_k_slice, d_period);

    let mut d_line = vec![f64::NAN; valid_start_idx];
    d_line.extend(valid_d_line);

    StochasticOutput {
        k_line,
        d_line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stochastic_oscillator() {
        // Simple uptrend
        let highs = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let lows = vec![9.0, 10.0, 11.0, 12.0, 13.0];
        let closes = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        
        let k_period = 3;
        let d_period = 2;

        let output = stochastic_oscillator(&highs, &lows, &closes, k_period, d_period);

        assert_eq!(output.k_line.len(), 5);
        assert_eq!(output.d_line.len(), 5);

        // Index 0, 1: NaN (k_period=3)
        assert!(output.k_line[1].is_nan());

        // Index 2: Highs[10,11,12], Lows[9,10,11], Close 12
        // HH=12, LL=9. K = (12-9)/(12-9)*100 = 100
        assert!((output.k_line[2] - 100.0).abs() < 1e-10);

        // Index 3: Highs[11,12,13], Lows[10,11,12], Close 13
        // HH=13, LL=10. K = (13-10)/(13-10)*100 = 100
        assert!((output.k_line[3] - 100.0).abs() < 1e-10);

        // D line: SMA(K, 2)
        // D[2] is NaN because valid K starts at 2, so SMA needs K[2] and K[1](NaN) -> wait, 
        // moving_average pads with NaNs.
        // valid_k_slice starts at index 2.
        // valid_d_line = moving_average([100, 100, 100], 2)
        // [NaN, 100, 100]
        // d_line = [NaN, NaN] + [NaN, 100, 100] = [NaN, NaN, NaN, 100, 100]
        
        assert!(output.d_line[2].is_nan());
        assert!((output.d_line[3] - 100.0).abs() < 1e-10);
    }
}
