use std::f64;

/// MACD (Moving Average Convergence Divergence) configuration
#[derive(Debug, Clone, Copy)]
pub struct MacdConfig {
    /// Fast EMA period (typically 12)
    pub fast_period: usize,
    /// Slow EMA period (typically 26)
    pub slow_period: usize,
    /// Signal line EMA period (typically 9)
    pub signal_period: usize,
}

impl Default for MacdConfig {
    fn default() -> Self {
        MacdConfig {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
        }
    }
}

/// MACD output values
#[derive(Debug, Clone)]
pub struct MacdOutput {
    /// MACD line (fast EMA - slow EMA)
    pub macd_line: Vec<f64>,
    /// Signal line (EMA of MACD line)
    pub signal_line: Vec<f64>,
    /// Histogram (MACD line - signal line)
    pub histogram: Vec<f64>,
}

/// Calculate Exponential Moving Average (EMA)
///
/// # Arguments
/// * `data` - Price data
/// * `period` - EMA period
///
/// # Returns
/// Vector of EMA values. First `period-1` values are NaN.
pub fn ema(data: &[f64], period: usize) -> Vec<f64> {
    if period == 0 || period > data.len() {
        return vec![f64::NAN; data.len()];
    }

    let mut ema_values = vec![f64::NAN; data.len()];
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Calculate initial SMA for the first EMA value
    let mut sum = 0.0;
    for i in 0..period {
        sum += data[i];
    }
    let initial_ema = sum / period as f64;
    ema_values[period - 1] = initial_ema;

    // Calculate subsequent EMA values
    for i in period..data.len() {
        ema_values[i] = (data[i] - ema_values[i - 1]) * multiplier + ema_values[i - 1];
    }

    ema_values
}

/// Calculate MACD indicator
///
/// # Arguments
/// * `data` - Price data (typically closing prices)
/// * `config` - MACD configuration (fast, slow, signal periods)
///
/// # Returns
/// MacdOutput containing MACD line, signal line, and histogram
pub fn macd(data: &[f64], config: MacdConfig) -> MacdOutput {
    let fast_ema = ema(data, config.fast_period);
    let slow_ema = ema(data, config.slow_period);

    // MACD line = fast EMA - slow EMA
    let macd_line: Vec<f64> = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(&fast, &slow)| {
            if fast.is_nan() || slow.is_nan() {
                f64::NAN
            } else {
                fast - slow
            }
        })
        .collect();

    // Signal line = EMA of MACD line
    // First, collect non-NaN MACD values for signal calculation
    let first_valid_idx = macd_line.iter().position(|&x| !x.is_nan()).unwrap_or(0);
    let mut signal_line = vec![f64::NAN; data.len()];
    
    if first_valid_idx < macd_line.len() {
        let valid_macd = &macd_line[first_valid_idx..];
        let signal_ema = ema(valid_macd, config.signal_period);
        
        for (i, &val) in signal_ema.iter().enumerate() {
            signal_line[first_valid_idx + i] = val;
        }
    }

    // Histogram = MACD line - signal line
    let histogram: Vec<f64> = macd_line
        .iter()
        .zip(signal_line.iter())
        .map(|(&macd, &signal)| {
            if macd.is_nan() || signal.is_nan() {
                f64::NAN
            } else {
                macd - signal
            }
        })
        .collect();

    MacdOutput {
        macd_line,
        signal_line,
        histogram,
    }
}

/// Calculate MACD histogram only (most commonly used for trading signals)
///
/// # Arguments
/// * `data` - Price data
/// * `config` - MACD configuration
///
/// # Returns
/// Vector of histogram values (MACD line - signal line)
pub fn macd_histogram(data: &[f64], config: MacdConfig) -> Vec<f64> {
    macd(data, config).histogram
}

/// Calculate MACD with default parameters (12, 26, 9)
///
/// # Arguments
/// * `data` - Price data
///
/// # Returns
/// MacdOutput with default configuration
pub fn macd_default(data: &[f64]) -> MacdOutput {
    macd(data, MacdConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let period = 3;
        let ema_vals = ema(&data, period);

        assert_eq!(ema_vals.len(), data.len());
        
        // First period-1 values should be NaN
        assert!(ema_vals[0].is_nan());
        assert!(ema_vals[1].is_nan());
        
        // First EMA value should be SMA
        assert!((ema_vals[2] - 2.0).abs() < 1e-10); // (1+2+3)/3 = 2.0
        
        // Subsequent values should be valid
        assert!(!ema_vals[3].is_nan());
        assert!(ema_vals[9] > ema_vals[2]); // Should increase with increasing data
    }

    #[test]
    fn test_macd_default() {
        // Create sample price data (uptrend)
        let prices: Vec<f64> = (0..100).map(|i| 100.0 + i as f64).collect();
        let output = macd_default(&prices);

        assert_eq!(output.macd_line.len(), prices.len());
        assert_eq!(output.signal_line.len(), prices.len());
        assert_eq!(output.histogram.len(), prices.len());

        // Early values should be NaN (need at least slow_period values)
        assert!(output.macd_line[0].is_nan());
        assert!(output.signal_line[0].is_nan());
        assert!(output.histogram[0].is_nan());

        // Later values should be valid
        let last_idx = prices.len() - 1;
        assert!(!output.macd_line[last_idx].is_nan());
        assert!(!output.signal_line[last_idx].is_nan());
        assert!(!output.histogram[last_idx].is_nan());

        // In an uptrend, MACD should generally be positive
        assert!(output.macd_line[last_idx] > 0.0);
    }

    #[test]
    fn test_macd_custom_config() {
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64).sin()).collect();
        let config = MacdConfig {
            fast_period: 5,
            slow_period: 10,
            signal_period: 3,
        };
        let output = macd(&prices, config);

        assert_eq!(output.macd_line.len(), prices.len());
        
        // Should have valid values after slow_period + signal_period
        let check_idx = config.slow_period + config.signal_period;
        if check_idx < prices.len() {
            assert!(!output.histogram[check_idx].is_nan());
        }
    }

    #[test]
    fn test_macd_histogram() {
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let hist = macd_histogram(&prices, MacdConfig::default());
        
        assert_eq!(hist.len(), prices.len());
        
        // Should have some valid values
        let valid_count = hist.iter().filter(|&&x| !x.is_nan()).count();
        assert!(valid_count > 0);
    }

    #[test]
    fn test_macd_crossover() {
        // Create data that crosses: down then up
        let mut prices = vec![100.0; 50];
        for i in 0..25 {
            prices[i] = 100.0 - i as f64; // Downtrend
        }
        for i in 25..50 {
            prices[i] = 75.0 + (i - 25) as f64; // Uptrend
        }

        let output = macd_default(&prices);
        
        // Find first valid histogram value
        let first_valid = output.histogram.iter().position(|&x| !x.is_nan()).unwrap();
        
        // Histogram should change sign (crossover)
        let mid_idx = (first_valid + 49) / 2;
        if mid_idx < output.histogram.len() - 5 {
            let early_hist = output.histogram[mid_idx];
            let late_hist = output.histogram[output.histogram.len() - 1];
            
            // Should have different signs or at least different values
            assert_ne!(early_hist, late_hist);
        }
    }
}
