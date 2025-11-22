use matlib::find_slope;

/// Calculates the Simple Moving Average (SMA) for a given data slice and number of lags.
///
/// # Arguments
///
/// * `data` - A slice of f64 values.
/// * `lags` - The window size for the moving average.
///
/// # Returns
///
/// A Vec<f64> containing the SMA values. The first `lags - 1` values are NaN.
pub fn moving_average(data: &[f64], lags: usize) -> Vec<f64> {
    if lags == 0 || lags > data.len() {
        return vec![f64::NAN; data.len()];
    }

    let mut sma = Vec::with_capacity(data.len());
    
    // Pad with NaN for the initial period where we don't have enough data
    for _ in 0..lags - 1 {
        sma.push(f64::NAN);
    }

    let mut sum: f64 = data.iter().take(lags).sum();
    sma.push(sum / lags as f64);

    for i in lags..data.len() {
        sum = sum - data[i - lags] + data[i];
        sma.push(sum / lags as f64);
    }

    sma
}

pub fn compute_trend(
    closes: &[f64],
    lookback: usize,
    full_lookback: usize,
    version: usize,
) -> Vec<f64> {
    let nprices = closes.len();
    let nind = nprices - full_lookback + 1;
    let mut trend = vec![0.0; nind];

    for (i, trd) in trend.iter_mut().enumerate().take(nind) {
        let k = full_lookback - 1 + i;
        *trd = match version {
            0 => find_slope(lookback, closes, k),
            1 => find_slope(lookback, closes, k) - find_slope(lookback, closes, k - lookback),
            _ => find_slope(lookback, closes, k) - find_slope(full_lookback, closes, k),
        };
    }

    trend
}

/// Compute moving average crossover indicators
pub fn compute_indicators(
    nind: usize,
    prices: &[f64],
    start_idx: usize,
    short_term: usize,
    long_term: usize,
) -> Vec<f64> {
    let mut inds = vec![0.0; nind];
    
    for i in 0..nind {
        let k = start_idx + i;
        
        // Compute short-term mean
        let mut short_mean = 0.0;
        for j in 0..short_term {
            short_mean += prices[k - j];
        }
        short_mean /= short_term as f64;
        
        // Compute long-term mean
        let mut long_mean = 0.0;
        for j in 0..long_term {
            long_mean += prices[k - j];
        }
        long_mean /= long_term as f64;
        
        inds[i] = short_mean - long_mean;
    }
    
    inds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moving_average() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let lags = 3;
        let sma = moving_average(&data, lags);

        assert_eq!(sma.len(), 5);
        assert!(sma[0].is_nan());
        assert!(sma[1].is_nan());
        assert!((sma[2] - 2.0).abs() < 1e-10); // (1+2+3)/3 = 2
        assert!((sma[3] - 3.0).abs() < 1e-10); // (2+3+4)/3 = 3
        assert!((sma[4] - 4.0).abs() < 1e-10); // (3+4+5)/3 = 4
    }

    #[test]
    fn test_moving_average_edge_cases() {
        let data = vec![1.0, 2.0];
        let lags = 3;
        let sma = moving_average(&data, lags);
        assert_eq!(sma.len(), 2);
        assert!(sma[0].is_nan());
        assert!(sma[1].is_nan());

        let lags = 0;
        let sma = moving_average(&data, lags);
        assert!(sma[0].is_nan());
    }
}
