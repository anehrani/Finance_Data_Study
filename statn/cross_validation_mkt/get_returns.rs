/// Computes one-bar returns for all short-term and long-term lookbacks
/// of a primitive moving-average crossover system.
/// 
/// The computed returns matrix has max_lookback * (max_lookback-1) / 2 rows
/// and nprices-max_lookback columns, which change fastest.
/// 
/// # Arguments
/// * `prices` - Log prices
/// * `max_lookback` - Maximum lookback to use
/// 
/// # Returns
/// A vector representing the returns matrix, organized as:
/// - n_systems rows (one per short/long lookback combination)
/// - n_returns columns (one per decision bar)
/// - Data is stored row-major: returns[system * n_returns + bar]
pub fn get_returns(prices: &[f64], max_lookback: usize) -> Vec<f64> {
    let nprices = prices.len();
    let n_returns = nprices.saturating_sub(max_lookback);
    let n_systems = max_lookback * (max_lookback - 1) / 2;
    
    let mut returns = vec![0.0; n_systems * n_returns];
    let mut iret = 0;
    
    // For each long-term lookback
    for ilong in 2..=max_lookback {
        // For each short-term lookback (must be less than long-term)
        for ishort in 1..ilong {
            // Compute short-term and long-term moving averages
            // The index of the first legal bar in prices is max_lookback-1
            // We must stop one bar before the end to compute the return
            
            let mut short_sum = 0.0;
            let mut long_sum = 0.0;
            
            for i in (max_lookback - 1)..(nprices - 1) {
                if i == max_lookback - 1 {
                    // Initialize sums for the first valid case
                    // Following C++ logic: for (j=i ; j>i-ishort ; j--)
                    short_sum = 0.0;
                    let mut j = i;
                    let short_limit = i.saturating_sub(ishort);
                    while j > short_limit {
                        short_sum += prices[j];
                        j -= 1;
                    }
                    
                    // long_sum starts with short_sum, then adds remaining elements
                    // Following C++ logic: while (j>i-ilong)
                    long_sum = short_sum;
                    let long_limit = i.saturating_sub(ilong);
                    while j > long_limit {
                        long_sum += prices[j];
                        j -= 1;
                    }
                } else {
                    // Update the moving averages
                    short_sum += prices[i] - prices[i - ishort];
                    long_sum += prices[i] - prices[i - ilong];
                }
                
                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;
                
                // Determine position and compute return
                let ret = if short_mean > long_mean {
                    // Long position
                    prices[i + 1] - prices[i]
                } else if short_mean < long_mean {
                    // Short position
                    prices[i] - prices[i + 1]
                } else {
                    // No position
                    0.0
                };
                
                returns[iret] = ret;
                iret += 1;
            }
        }
    }
    
    assert_eq!(iret, n_systems * n_returns);
    returns
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_returns_size() {
        let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let max_lookback = 5;
        let returns = get_returns(&prices, max_lookback);
        
        let n_systems = max_lookback * (max_lookback - 1) / 2; // 5*4/2 = 10
        let n_returns = prices.len() - max_lookback; // 10 - 5 = 5
        
        assert_eq!(returns.len(), n_systems * n_returns);
    }
    
    #[test]
    fn test_get_returns_basic() {
        // Simple trending prices
        let prices: Vec<f64> = (0..20).map(|i| (i as f64).ln()).collect();
        let max_lookback = 3;
        let returns = get_returns(&prices, max_lookback);
        
        // Should have 3*2/2 = 3 systems
        // Should have 20-3 = 17 returns per system
        assert_eq!(returns.len(), 3 * 17);
    }
}
