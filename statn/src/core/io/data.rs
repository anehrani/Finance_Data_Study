/// Training and test data split
#[derive(Debug, Clone)]
pub struct DataSplit {
    pub train_data: Vec<f64>,
    pub test_data: Vec<f64>,
    pub max_lookback: usize,
}

/// Split data into training and test sets with lookback
/// 
/// # Arguments
/// * `data` - Input data (typically log prices)
/// * `max_lookback` - Maximum lookback period needed for indicators
/// * `n_test` - Number of test cases
/// 
/// # Returns
/// DataSplit with training and test data, ensuring test data has enough lookback
pub fn split_train_test(
    data: &[f64],
    max_lookback: usize,
    n_test: usize,
) -> Result<DataSplit, String> {
    // We need:
    // - max_lookback prices for initial lookback
    // - n_test prices for test cases
    // - 1 extra price to compute the last target return (price[n_test] - price[n_test-1])
    let total_needed = max_lookback + n_test + 1;
    
    if data.len() < total_needed {
        return Err(format!(
            "Insufficient data: need at least {} prices, got {}",
            total_needed, data.len()
        ));
    }
    
    // Calculate how many training cases we can have
    // Total data = max_lookback + n_train + 1 (for last train target) + n_test + 1 (for last test target)
    // But we share the lookback between train and test, so:
    // data.len() = max_lookback + n_train + 1 + n_test + 1
    // n_train = data.len() - max_lookback - n_test - 2
    let n_train = data.len() - max_lookback - n_test - 1;
    
    // Training data: from start to (max_lookback + n_train + 1)
    // The +1 is for computing the last training target
    let train_end = max_lookback + n_train + 1;
    let train_data = data[..train_end].to_vec();
    
    // Test data: from (train_end - max_lookback - 1) to end
    // We need max_lookback for indicators, plus n_test + 1 for targets
    let test_start = train_end - max_lookback - 1;
    let test_data = data[test_start..].to_vec();
    
    Ok(DataSplit {
        train_data,
        test_data,
        max_lookback,
    })
}

/// Compute target returns from prices
/// 
/// # Arguments
/// * `prices` - Price data (typically log prices)
/// * `start_idx` - Starting index in the price array
/// * `n_cases` - Number of target returns to compute
/// 
/// # Returns
/// Vector of returns (price[i+1] - price[i])
pub fn compute_targets(prices: &[f64], start_idx: usize, n_cases: usize) -> Vec<f64> {
    (0..n_cases)
        .map(|i| {
            let idx = start_idx + i;
            prices[idx + 1] - prices[idx]
        })
        .collect()
}

/// Compute simple returns from prices
/// 
/// # Arguments
/// * `prices` - Raw price data (not log-transformed)
/// 
/// # Returns
/// Vector of simple returns: (price[i+1] - price[i]) / price[i]
pub fn compute_returns(prices: &[f64]) -> Vec<f64> {
    prices.windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect()
}

/// Compute log returns from log prices
/// 
/// # Arguments
/// * `log_prices` - Log-transformed price data
/// 
/// # Returns
/// Vector of log returns: log_price[i+1] - log_price[i]
pub fn compute_log_returns(log_prices: &[f64]) -> Vec<f64> {
    log_prices.windows(2)
        .map(|w| w[1] - w[0])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_split_train_test() {
        let prices: Vec<f64> = (0..1000).map(|i| (100.0 + i as f64).ln()).collect();
        let split = split_train_test(&prices, 200, 252).unwrap();
        
        assert_eq!(split.max_lookback, 200);
        assert!(split.train_data.len() > 0);
        // Test data needs max_lookback + n_test + 1 for computing last target
        assert_eq!(split.test_data.len(), 200 + 252 + 1);
        
        // Verify we can compute all targets
        let n_train = prices.len() - 200 - 252 - 1;
        assert_eq!(split.train_data.len(), 200 + n_train + 1);
    }
    
    #[test]
    fn test_split_insufficient_data() {
        let prices = vec![1.0, 2.0, 3.0];
        let result = split_train_test(&prices, 100, 100);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient data"));
    }
    
    #[test]
    fn test_compute_targets() {
        let prices = vec![1.0, 1.1, 1.05, 1.15];
        let targets = compute_targets(&prices, 0, 3);
        
        assert_eq!(targets.len(), 3);
        assert!((targets[0] - 0.1).abs() < 1e-10);
        assert!((targets[1] - (-0.05)).abs() < 1e-10);
        assert!((targets[2] - 0.1).abs() < 1e-10);
    }
    
    #[test]
    fn test_compute_returns() {
        let prices = vec![100.0, 110.0, 105.0, 115.5];
        let returns = compute_returns(&prices);
        
        assert_eq!(returns.len(), 3);
        assert!((returns[0] - 0.1).abs() < 1e-10);  // (110-100)/100 = 0.1
        assert!((returns[1] - (-0.045454545)).abs() < 1e-6);  // (105-110)/110
    }
    
    #[test]
    fn test_compute_log_returns() {
        let log_prices = vec![1.0, 1.1, 1.05, 1.15];
        let returns = compute_log_returns(&log_prices);
        
        assert_eq!(returns.len(), 3);
        assert!((returns[0] - 0.1).abs() < 1e-10);
        assert!((returns[1] - (-0.05)).abs() < 1e-10);
        assert!((returns[2] - 0.1).abs() < 1e-10);
    }
}
