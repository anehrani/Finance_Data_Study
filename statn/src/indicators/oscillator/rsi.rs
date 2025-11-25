/// Compute RSI (Relative Strength Index) indicator
///
/// RSI is a momentum oscillator that measures the speed and magnitude of price changes.
/// Standard formula:
/// - Calculate price changes (gains and losses)
/// - Compute exponential moving average of gains and losses
/// - RS = Average Gain / Average Loss
/// - RSI = 100 - (100 / (1 + RS))
///
/// # Arguments
/// * `n_cases` - Number of cases to compute
/// * `prices` - Price data (log prices)
/// * `start_idx` - Starting index in prices array
/// * `period` - RSI lookback period
///
/// # Returns
/// Vector of RSI values normalized to [-1, 1] range:
/// - RSI > 70 (overbought) → positive values approaching 1
/// - RSI < 30 (oversold) → negative values approaching -1
/// - RSI = 50 (neutral) → 0
pub fn compute_rsi(
    n_cases: usize,
    prices: &[f64],
    start_idx: usize,
    period: usize,
) -> Vec<f64> {
    let mut rsi_values = vec![0.0; n_cases];
    
    if period == 0 || start_idx + n_cases > prices.len() {
        return rsi_values;
    }
    
    // We need at least period+1 prices before start_idx to compute RSI
    if start_idx < period {
        return rsi_values;
    }
    
    for (i, rsi_val) in rsi_values.iter_mut().enumerate().take(n_cases) {
        let current_idx = start_idx + i;
        
        // Need period+1 prices to compute period changes
        if current_idx < period {
            continue;
        }
        
        // Calculate initial average gain and loss using SMA for first period
        let mut avg_gain = 0.0;
        let mut avg_loss = 0.0;
        
        // Compute gains and losses for the initial period
        for j in 1..=period {
            let change = prices[current_idx - period + j] - prices[current_idx - period + j - 1];
            if change > 0.0 {
                avg_gain += change;
            } else {
                avg_loss -= change; // Store as positive value
            }
        }
        
        avg_gain /= period as f64;
        avg_loss /= period as f64;
        
        // Calculate RS and RSI
        let rsi = if avg_gain < 1e-10 && avg_loss < 1e-10 {
            // No price movement - neutral RSI
            50.0
        } else if avg_loss < 1e-10 {
            // Only gains, no losses - RSI = 100
            100.0
        } else {
            let rs = avg_gain / avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        };
        
        // Normalize RSI from [0, 100] to [-1, 1]
        // RSI = 50 → 0
        // RSI = 100 → 1
        // RSI = 0 → -1
        *rsi_val = (rsi - 50.0) / 50.0;
    }
    
    rsi_values
}

/// Compute RSI indicator with exponential smoothing (Wilder's method)
///
/// This is the more common implementation using exponential moving average
/// for smoothing gains and losses after the initial period.
///
/// # Arguments
/// * `n_cases` - Number of cases to compute
/// * `prices` - Price data (log prices)
/// * `start_idx` - Starting index in prices array
/// * `period` - RSI lookback period (typically 14)
///
/// # Returns
/// Vector of RSI values normalized to [-1, 1] range
pub fn compute_rsi_ema(
    n_cases: usize,
    prices: &[f64],
    start_idx: usize,
    period: usize,
) -> Vec<f64> {
    let mut rsi_values = vec![0.0; n_cases];
    
    if period == 0 || start_idx + n_cases > prices.len() {
        return rsi_values;
    }
    
    // We need at least period+1 prices before start_idx to initialize
    if start_idx < period {
        return rsi_values;
    }
    
    // Calculate initial average gain and loss using SMA
    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;
    
    for j in 1..=period {
        let change = prices[start_idx - period + j] - prices[start_idx - period + j - 1];
        if change > 0.0 {
            avg_gain += change;
        } else {
            avg_loss -= change;
        }
    }
    
    avg_gain /= period as f64;
    avg_loss /= period as f64;
    
    // Wilder's smoothing factor
    let alpha = 1.0 / period as f64;
    
    // Compute RSI for each case using EMA
    for (i, rsi_val) in rsi_values.iter_mut().enumerate().take(n_cases) {
        let current_idx = start_idx + i;
        
        if current_idx == 0 {
            continue;
        }
        
        // Calculate current price change
        let change = prices[current_idx] - prices[current_idx - 1];
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { -change } else { 0.0 };
        
        // Update exponential moving averages (Wilder's smoothing)
        avg_gain = alpha * gain + (1.0 - alpha) * avg_gain;
        avg_loss = alpha * loss + (1.0 - alpha) * avg_loss;
        
        // Calculate RS and RSI
        let rsi = if avg_gain < 1e-10 && avg_loss < 1e-10 {
            // No price movement - neutral RSI
            50.0
        } else if avg_loss < 1e-10 {
            // Only gains, no losses - RSI = 100
            100.0
        } else {
            let rs = avg_gain / avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        };
        
        // Normalize to [-1, 1]
        *rsi_val = (rsi - 50.0) / 50.0;
    }
    
    rsi_values
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rsi_basic() {
        // Create a simple price series with clear trend
        let prices = vec![
            1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9,
            2.0, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9,
        ];
        
        let period = 14;
        let n_cases = 5;
        let start_idx = 15;
        
        let rsi = compute_rsi(n_cases, &prices, start_idx, period);
        
        // With consistent upward trend, RSI should be high (positive)
        assert_eq!(rsi.len(), n_cases);
        for &val in &rsi {
            assert!(val > 0.0, "RSI should be positive for uptrend");
        }
    }
    
    #[test]
    fn test_rsi_downtrend() {
        // Create a downward trend
        let prices = vec![
            2.9, 2.8, 2.7, 2.6, 2.5, 2.4, 2.3, 2.2, 2.1, 2.0,
            1.9, 1.8, 1.7, 1.6, 1.5, 1.4, 1.3, 1.2, 1.1, 1.0,
        ];
        
        let period = 14;
        let n_cases = 5;
        let start_idx = 15;
        
        let rsi = compute_rsi(n_cases, &prices, start_idx, period);
        
        // With consistent downward trend, RSI should be low (negative)
        assert_eq!(rsi.len(), n_cases);
        for &val in &rsi {
            assert!(val < 0.0, "RSI should be negative for downtrend");
        }
    }
    
    #[test]
    fn test_rsi_neutral() {
        // Create a flat series
        let prices = vec![1.0; 30];
        
        let period = 14;
        let n_cases = 10;
        let start_idx = 15;
        
        let rsi = compute_rsi(n_cases, &prices, start_idx, period);
        
        // With no change, RSI should be near 0 (neutral)
        assert_eq!(rsi.len(), n_cases);
        for &val in &rsi {
            assert!(val.abs() < 0.1, "RSI should be near 0 for flat prices");
        }
    }
    
    #[test]
    fn test_rsi_ema_basic() {
        let prices = vec![
            1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9,
            2.0, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9,
        ];
        
        let period = 14;
        let n_cases = 5;
        let start_idx = 15;
        
        let rsi = compute_rsi_ema(n_cases, &prices, start_idx, period);
        
        assert_eq!(rsi.len(), n_cases);
        for &val in &rsi {
            assert!(val > 0.0, "RSI EMA should be positive for uptrend");
        }
    }
    
    #[test]
    fn test_rsi_normalization() {
        // Test that normalization works correctly
        let prices = vec![1.0; 30];
        
        let period = 14;
        let n_cases = 10;
        let start_idx = 15;
        
        let rsi = compute_rsi(n_cases, &prices, start_idx, period);
        
        // All values should be in [-1, 1] range
        for &val in &rsi {
            assert!(val >= -1.0 && val <= 1.0, "RSI should be normalized to [-1, 1]");
        }
    }
}
