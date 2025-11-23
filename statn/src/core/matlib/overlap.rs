
/*
Compute a single indicator (linear slope of a price block) 
and a single target (price change over a specified lookahead)
*/

pub fn ind_targ(
    lookback: usize,
    lookahead: usize,
    x: &[f64],
    x_idx: usize, // Index into x array for current price
) -> (f64, f64) {
    let start_idx = if x_idx >= lookback - 1 {
        x_idx - lookback + 1
    } else {
        0
    };

    let mut slope = 0.0;
    let mut denom = 0.0;

    for i in 0..lookback {
        let coef = 2.0 * i as f64 / (lookback - 1) as f64 - 1.0;
        denom += coef * coef;
        slope += coef * x[start_idx + i];
    }

    let indicator = slope / denom;
    let target = x[x_idx + lookahead] - x[x_idx];
    (indicator, target)
}

/*
Compute beta coefficient for simple linear regression
*/

pub fn find_beta(data: &[(f64, f64)]) -> (f64, f64) {
    let ntrn = data.len();
    if ntrn == 0 {
        return (0.0, 0.0);
    }

    let mut xmean = 0.0;
    let mut ymean = 0.0;

    for &(x, y) in data {
        xmean += x;
        ymean += y;
    }

    xmean /= ntrn as f64;
    ymean /= ntrn as f64;

    let mut xy = 0.0;
    let mut xx = 0.0;

    for &(x, y) in data {
        let dx = x - xmean;
        let dy = y - ymean;
        xy += dx * dy;
        xx += dx * dx;
    }

    let beta = xy / (xx + 1e-60);
    let constant = ymean - beta * xmean;
    (beta, constant)
}
