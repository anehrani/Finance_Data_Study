

/*
--------------------------------------------------------------------------------
   Local routine computes linear slope (trend)
--------------------------------------------------------------------------------
*/
pub fn find_slope(lookback: usize, x: &[f64], index: usize) -> f64 {
    let start = if index >= lookback - 1 {
        index + 1 - lookback
    } else {
        0
    };

    let mut slope = 0.0;
    let mut denom = 0.0;

    for i in 0..lookback {
        let coef = i as f64 - 0.5 * (lookback - 1) as f64;
        denom += coef * coef;
        slope += coef * x[start + i];
    }

    slope / denom
}


/*
Compute range expansion (bad indicator for demo only)
*/

pub fn range_expansion(lookback: usize, x_ptr: usize, close: &[f64]) -> f64 {
    let start_idx = if x_ptr >= lookback - 1 {
        x_ptr + 1 - lookback
    } else {
        0
    };

    let mut recent_high = -1e60;
    let mut recent_low = 1e60;
    let mut older_high = -1e60;
    let mut older_low = 1e60;

    for i in 0..(lookback / 2) {
        if close[start_idx + i] > older_high {
            older_high = close[start_idx + i];
        }
        if close[start_idx + i] < older_low {
            older_low = close[start_idx + i];
        }
    }

    for i in (lookback / 2)..lookback {
        if close[start_idx + i] > recent_high {
            recent_high = close[start_idx + i];
        }
        if close[start_idx + i] < recent_low {
            recent_low = close[start_idx + i];
        }
    }

    (recent_high - recent_low) / (older_high - older_low + 1e-10)
}


/*
Compute price jump
*/

pub fn jump(lookback: usize, x_ptr: usize, close: &[f64]) -> f64 {
    let start_idx = if x_ptr >= lookback - 1 {
        x_ptr + 1 - lookback
    } else {
        0
    };

    let alpha = 2.0 / lookback as f64;
    let mut smoothed = close[start_idx];

    for i in 1..(lookback - 1) {
        smoothed = alpha * close[start_idx + i] + (1.0 - alpha) * smoothed;
    }

    close[start_idx + lookback - 1] - smoothed
}


