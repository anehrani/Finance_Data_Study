
/// Evaluate a thresholded moving-average crossover system
pub fn test_system(
    prices: &[f64],
    max_lookback: usize,
    long_term: usize,
    short_pct: f64,
    short_thresh: f64,
    long_thresh: f64,
    returns: Option<&mut [f64]>,
) -> (f64, i32) {
    let ncases = prices.len();
    let short_term = (0.01 * short_pct * long_term as f64) as usize;
    let short_term = short_term.max(1).min(long_term - 1);
    
    let short_thresh = short_thresh / 10000.0;
    let long_thresh = long_thresh / 10000.0;

    let mut sum = 0.0;
    let mut ntrades = 0;
    
    let process_trade = |i: usize, prices: &[f64], short_mean: f64, long_mean: f64| -> (f64, bool) {
        let change = short_mean / long_mean - 1.0;
        if change > long_thresh {
            (prices[i+1] - prices[i], true)
        } else if change < -short_thresh {
            (prices[i] - prices[i+1], true)
        } else {
            (0.0, false)
        }
    };
    
    match returns {
        Some(ret_slice) => {
            let mut ret_idx = 0;
            for i in (max_lookback - 1)..(ncases - 1) {
                let short_mean: f64 = prices[i + 1 - short_term..=i].iter().sum::<f64>() / short_term as f64;
                let long_mean: f64 = prices[i + 1 - long_term..=i].iter().sum::<f64>() / long_term as f64;
                
                let (ret, traded) = process_trade(i, prices, short_mean, long_mean);
                if traded { ntrades += 1; }
                sum += ret;
                if ret_idx < ret_slice.len() {
                    ret_slice[ret_idx] = ret;
                    ret_idx += 1;
                }
            }
        }
        None => {
            for i in (max_lookback - 1)..(ncases - 1) {
                let short_mean: f64 = prices[i + 1 - short_term..=i].iter().sum::<f64>() / short_term as f64;
                let long_mean: f64 = prices[i + 1 - long_term..=i].iter().sum::<f64>() / long_term as f64;
                
                let (ret, traded) = process_trade(i, prices, short_mean, long_mean);
                if traded { ntrades += 1; }
                sum += ret;
            }
        }
    }

    (sum, ntrades)
}

