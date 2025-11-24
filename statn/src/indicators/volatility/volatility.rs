use finance_tools::atr;

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

    for (i, vlt) in volatility.iter_mut().enumerate().take(nind) {
        let k = full_lookback - 1 + i;
        *vlt = match version {
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
