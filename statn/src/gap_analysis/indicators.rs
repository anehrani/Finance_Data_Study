use matlib::find_slope;
use finance_tools::atr;

pub fn compute_trend(
    closes: &[f64],
    lookback: usize,
    full_lookback: usize,
    version: usize,
) -> Vec<f64> {
    let nprices = closes.len();
    let nind = nprices - full_lookback + 1;
    let mut trend = vec![0.0; nind];

    for i in 0..nind {
        let k = full_lookback - 1 + i;
        trend[i] = match version {
            0 => find_slope(lookback, closes, k),
            1 => find_slope(lookback, closes, k) - find_slope(lookback, closes, k - lookback),
            _ => find_slope(lookback, closes, k) - find_slope(full_lookback, closes, k),
        };
    }

    trend
}

pub fn compute_volatility(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    lookback: usize,
    full_lookback: usize,
    version: usize,
) -> Vec<f64> {
    let nprices = closes.len();
    let nind = nprices - full_lookback + 1;
    let mut volatility = vec![0.0; nind];

    for i in 0..nind {
        let k = full_lookback - 1 + i;
        volatility[i] = match version {
            0 => atr(lookback, highs, lows, closes, k),
            1 => {
                atr(lookback, highs, lows, closes, k)
                    - atr(lookback, highs, lows, closes, k - lookback)
            }
            _ => {
                atr(lookback, highs, lows, closes, k)
                    - atr(full_lookback, highs, lows, closes, k)
            }
        };
    }

    volatility
}

pub fn find_quantile(sorted_data: &[f64], fractile: f64) -> f64 {
    let n = sorted_data.len();
    let mut k = ((fractile * (n as f64 + 1.0)) as usize).saturating_sub(1);
    if k >= n {
        k = n - 1;
    }
    sorted_data[k]
}

pub fn find_min_max(data: &[f64]) -> (f64, f64) {
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;

    for &val in data {
        if val < min_val {
            min_val = val;
        }
        if val > max_val {
            max_val = val;
        }
    }

    (min_val, max_val)
}