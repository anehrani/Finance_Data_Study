/// Tests a trained crossover system
/// Computes the mean return
pub fn test_system(
    long_v_short: bool,  // true = long-only, false = short-only
    x: &[f64],
    short_term: usize,
    long_term: usize
) -> f64 {
    let ncases = x.len();
    let mut sum = 0.0;
    let mut n_trades = 0;
    
    for i in (long_term - 1)..(ncases - 1) {
        // short-term mean
        let mut short_sum = 0.0;
        for j in 0..short_term {
            short_sum += x[i - j];
        }
        let short_mean = short_sum / short_term as f64;
        // long-term mean
        let mut long_sum = short_sum;
        for j in short_term..long_term {
            long_sum += x[i - j];
        }
        let long_mean = long_sum / long_term as f64;
        
        // Only trade in the specified direction
        if long_v_short && short_mean > long_mean {
            // Long position
            sum += x[i + 1] - x[i];
            n_trades += 1;
        } else if !long_v_short && short_mean < long_mean {
            // Short position
            sum -= x[i + 1] - x[i];
            n_trades += 1;
        }
    }
    sum / (n_trades as f64 + 1.0e-30)
}

/// Wrapper for backward compatibility - trades both directions (original TrnBias behavior)
pub fn test_system_both_directions(
    x: &[f64],
    short_term: usize,
    long_term: usize
) -> f64 {
    let ncases = x.len();
    let mut sum = 0.0;
    
    for i in (long_term - 1)..(ncases - 1) {
        let mut short_sum = 0.0;
        for j in 0..short_term {
            short_sum += x[i - j];
        }
        let short_mean = short_sum / short_term as f64;
        
        let mut long_sum = short_sum;
        for j in short_term..long_term {
            long_sum += x[i - j];
        }
        let long_mean = long_sum / long_term as f64;
        
        // Trade both directions
        if short_mean > long_mean {
            sum += x[i + 1] - x[i];
        } else if short_mean < long_mean {
            sum -= x[i + 1] - x[i];
        }
    }
    sum / (ncases - long_term) as f64
}
