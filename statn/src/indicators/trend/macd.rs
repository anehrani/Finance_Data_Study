use crate::trend::ma::exponential_moving_average;

/// MACD (Moving Average Convergence Divergence) Output
#[derive(Debug, Clone)]
pub struct MacdOutput {
    /// MACD Line = Fast EMA - Slow EMA
    pub macd_line: Vec<f64>,
    /// Signal Line = EMA of MACD Line
    pub signal_line: Vec<f64>,
    /// Histogram = MACD Line - Signal Line
    pub histogram: Vec<f64>,
}

/// Computes the MACD indicator.
///
/// # Arguments
///
/// * `data` - A slice of f64 values (typically closing prices).
/// * `fast_period` - The lookback period for the fast EMA (typically 12).
/// * `slow_period` - The lookback period for the slow EMA (typically 26).
/// * `signal_period` - The lookback period for the signal line EMA (typically 9).
///
/// # Returns
///
/// A `MacdOutput` struct containing the MACD line, signal line, and histogram.
pub fn macd(
    data: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> MacdOutput {
    let fast_ema = exponential_moving_average(data, fast_period);
    let slow_ema = exponential_moving_average(data, slow_period);

    let mut macd_line = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        if i < slow_period - 1 {
            macd_line.push(f64::NAN);
        } else {
            macd_line.push(fast_ema[i] - slow_ema[i]);
        }
    }

    // Calculate signal line (EMA of MACD line)
    // We need to handle the initial NaNs in macd_line.
    // exponential_moving_average handles NaNs by propagating them or starting late?
    // My implementation of EMA pads with NaNs. If input has NaNs, the sum will be NaN.
    // So we need to slice the valid part of macd_line.
    
    let valid_start_idx = slow_period - 1;
    if valid_start_idx >= macd_line.len() {
        return MacdOutput {
            macd_line,
            signal_line: vec![f64::NAN; data.len()],
            histogram: vec![f64::NAN; data.len()],
        };
    }

    let valid_macd_slice = &macd_line[valid_start_idx..];
    let valid_signal_line = exponential_moving_average(valid_macd_slice, signal_period);

    let mut signal_line = vec![f64::NAN; valid_start_idx];
    signal_line.extend(valid_signal_line);

    let mut histogram = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        if i < valid_start_idx + signal_period - 1 {
            histogram.push(f64::NAN);
        } else {
            histogram.push(macd_line[i] - signal_line[i]);
        }
    }

    MacdOutput {
        macd_line,
        signal_line,
        histogram,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macd_basic() {
        let data = vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0,
            11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0,
            21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0,
        ];
        
        // Use small periods for testing
        let fast = 3;
        let slow = 5;
        let signal = 3;

        let output = macd(&data, fast, slow, signal);

        assert_eq!(output.macd_line.len(), 30);
        assert_eq!(output.signal_line.len(), 30);
        assert_eq!(output.histogram.len(), 30);

        // Check NaNs
        // MACD line valid from index 4 (slow-1)
        assert!(output.macd_line[3].is_nan());
        assert!(!output.macd_line[4].is_nan());

        // Signal line valid from index 4 + 2 (signal-1) = 6
        assert!(output.signal_line[5].is_nan());
        assert!(!output.signal_line[6].is_nan());
    }
}
