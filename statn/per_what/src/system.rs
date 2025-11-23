

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationCriterion {
    MeanReturn = 0,
    ProfitFactor = 1,
    SharpeRatio = 2,
}

impl From<i32> for OptimizationCriterion {
    fn from(v: i32) -> Self {
        match v {
            0 => OptimizationCriterion::MeanReturn,
            1 => OptimizationCriterion::ProfitFactor,
            2 => OptimizationCriterion::SharpeRatio,
            _ => OptimizationCriterion::ProfitFactor, // Default
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReturnType {
    AllBars = 0,
    OpenPosition = 1,
    CompletedTrades = 2,
}

impl From<i32> for ReturnType {
    fn from(v: i32) -> Self {
        match v {
            0 => ReturnType::AllBars,
            1 => ReturnType::OpenPosition,
            2 => ReturnType::CompletedTrades,
            _ => ReturnType::CompletedTrades, // Default
        }
    }
}

/// Computes optimal lookback and breakout threshold
pub fn opt_params(
    which_crit: OptimizationCriterion,
    all_bars: bool,
    prices: &[f64],
    max_lookback: usize,
) -> (usize, f64, i32, Double) {
    let nprices = prices.len();
    let mut best_perf = -1.0e60;
    let mut ibestlook = 0;
    let mut ibestthresh = 0;
    let mut last_position_of_best = 0;

    for ilook in 2..=max_lookback {
        for ithresh in 1..=10 {
            let mut total_return = 0.0;
            let mut win_sum = 1.0e-60;
            let mut lose_sum = 1.0e-60;
            let mut sum_squares = 1.0e-60;
            let mut n_trades = 0;
            let mut position = 0;

            // We need at least ilook history for the first MA calculation.
            // But the loop in C++ starts at max_lookback-1 to make all trials comparable.
            
            let start_idx = max_lookback - 1;
            
            // We need to calculate MA ending at 'i'.
            // MA is sum(prices[i-ilook+1..=i]) / ilook?
            // C++:
            // if (i == max_lookback-1) {
            //    for (j=i ; j>i-ilook ; j--) MA_sum += prices[j] ;
            // }
            // This sums prices[i], prices[i-1] ... prices[i-ilook+1]. Correct.
            
            let mut ma_sum = 0.0;
            // Initialize MA sum for the first iteration
            for j in (start_idx + 1 - ilook)..=start_idx {
                 ma_sum += prices[j];
            }

            for i in start_idx..nprices - 1 {
                if i > start_idx {
                    ma_sum += prices[i] - prices[i - ilook];
                }

                let ma_mean = ma_sum / ilook as f64;
                let trial_thresh = 1.0 + 0.01 * ithresh as f64;

                if prices[i] > trial_thresh * ma_mean {
                    position = 1;
                } else if prices[i] < ma_mean {
                    position = 0;
                }

                let ret = if position == 1 {
                    prices[i + 1] - prices[i]
                } else {
                    0.0
                };

                if all_bars || position == 1 {
                    n_trades += 1;
                    total_return += ret;
                    sum_squares += ret * ret;
                    if ret > 0.0 {
                        win_sum += ret;
                    } else {
                        lose_sum -= ret;
                    }
                }
            }

            let perf = match which_crit {
                OptimizationCriterion::MeanReturn => total_return / (n_trades as f64 + 1.0e-30),
                OptimizationCriterion::ProfitFactor => win_sum / lose_sum,
                OptimizationCriterion::SharpeRatio => {
                    let mean_ret = total_return / (n_trades as f64 + 1.0e-30);
                    let mean_sq = sum_squares / (n_trades as f64 + 1.0e-30);
                    let variance = mean_sq - mean_ret * mean_ret;
                    let safe_variance = if variance < 1.0e-20 { 1.0e-20 } else { variance };
                    mean_ret / safe_variance.sqrt()
                }
            };

            if perf > best_perf {
                best_perf = perf;
                ibestlook = ilook;
                ibestthresh = ithresh;
                last_position_of_best = position;
            }
        }
    }

    (ibestlook, 0.01 * ibestthresh as f64, last_position_of_best, best_perf)
}




pub fn comp_return_full(
    ret_type: ReturnType,
    prices: &[f64],
    test_start_idx: usize, // Index of the first return in the test set
    n_test: usize,
    lookback: usize,
    thresh: f64,
    last_pos: i32,
) -> Vec<f64> {
    let mut returns = Vec::new();
    let mut position = last_pos;
    let mut prior_position = 0;
    let trial_thresh = 1.0 + thresh;
    let mut open_price = 0.0;
    
    // The loop in C++: for (i=istart-1 ; i<istart-1+ntest ; i++)
    // istart is `test_start_idx`.
    // i is the index of the bar where the decision is made.
    // The return is `prices[i+1] - prices[i]`.
    
    let start_decision_idx = test_start_idx - 1;
    let end_decision_idx = start_decision_idx + n_test;
    
    let mut ma_sum = 0.0;
    // Initialize MA for the first decision point
    for j in (start_decision_idx + 1 - lookback)..=start_decision_idx {
        ma_sum += prices[j];
    }

    for i in start_decision_idx..end_decision_idx {
        if i > start_decision_idx {
            ma_sum += prices[i] - prices[i - lookback];
        }
        
        let ma_mean = ma_sum / lookback as f64;
        
        if prices[i] > trial_thresh * ma_mean {
            position = 1;
        } else if prices[i] < ma_mean {
            position = 0;
        }
        
        let ret = if position == 1 {
            prices[i+1] - prices[i]
        } else {
            0.0
        };
        
        match ret_type {
            ReturnType::AllBars => returns.push(ret),
            ReturnType::OpenPosition => {
                if position == 1 {
                    returns.push(ret);
                }
            }
            ReturnType::CompletedTrades => {
                if position == 1 && prior_position == 0 {
                    open_price = prices[i];
                } else if prior_position == 1 && position == 0 {
                    returns.push(prices[i] - open_price);
                } else if position == 1 && i == end_decision_idx - 1 {
                    // Force close at end of data
                    returns.push(prices[i+1] - open_price);
                }
            }
        }
        
        prior_position = position;
    }
    
    returns
}

// Type alias for double to match C++ signature in my head, but Rust uses f64
type Double = f64;
