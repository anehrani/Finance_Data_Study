/*
--------------------------------------------------------------------------------
   Local routine computes average true range
--------------------------------------------------------------------------------
*/
pub fn atr(lookback: usize, high: &[f64], low: &[f64], close: &[f64], index: usize) -> f64 {
    let start = if index >= lookback - 1 {
        index + 1 - lookback
    } else {
        0
    };

    let mut sum = 0.0;

    for i in 0..lookback {
        let mut term = high[start + i] - low[start + i];

        if i > 0 {
            let gap1 = high[start + i] - close[start + i - 1];
            let gap2 = close[start + i - 1] - low[start + i];

            if gap1 > term {
                term = gap1;
            }
            if gap2 > term {
                term = gap2;
            }
        }
        sum += term;
    }

    sum / lookback as f64
}

