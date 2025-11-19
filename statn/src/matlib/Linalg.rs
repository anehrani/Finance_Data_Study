

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
