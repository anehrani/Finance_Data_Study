

use matlib::{qsortd, find_slope, range_expansion, jump};
use stats::entropy;
use statn::core::data_utils::chart::BarData;
use finance_tools::atr;

/*
Compute indicator statistics
*/

pub fn compute_indicator_stats(
    indicator: &[f64],
    name: &str,
    nbins: usize,
) {
    if indicator.is_empty() {
        return;
    }

    let mut sorted = indicator.to_vec();
    qsortd(0, sorted.len() - 1, &mut sorted);

    let minval = sorted[0];
    let maxval = sorted[sorted.len() - 1];
    let median = if sorted.len() % 2 == 1 {
        sorted[sorted.len() / 2]
    } else {
        0.5 * (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2])
    };

    let rel_entropy = entropy(indicator, nbins);

    println!(
        "\n{}  min={:.4}  max={:.4}  median={:.4}  relative entropy={:.3}",
        name, minval, maxval, median, rel_entropy
    );
}

pub fn calculate_trend(
    bars: &BarData,
    lookback: usize,
    full_lookback: usize,
    version: i32,
) -> Vec<f64> {
    let nprices = bars.len();
    let nind = nprices - full_lookback + 1;
    let mut trend = Vec::with_capacity(nind);

    for i in 0..nind {
        let k = full_lookback - 1 + i;
        let val = if version == 0 {
            find_slope(lookback, &bars.close, k)
        } else if version == 1 {
            find_slope(lookback, &bars.close, k)
                - find_slope(lookback, &bars.close, k.saturating_sub(lookback))
        } else {
            find_slope(lookback, &bars.close, k) - find_slope(full_lookback, &bars.close, k)
        };
        trend.push(val);
    }
    trend
}

pub fn calculate_volatility(
    bars: &BarData,
    lookback: usize,
    full_lookback: usize,
    version: i32,
) -> Vec<f64> {
    let nprices = bars.len();
    let nind = nprices - full_lookback + 1;
    let mut volatility = Vec::with_capacity(nind);

    for i in 0..nind {
        let k = full_lookback - 1 + i;
        let val = if version == 0 {
            atr(lookback, &bars.high, &bars.low, &bars.close, k)
        } else if version == 1 {
            atr(lookback, &bars.high, &bars.low, &bars.close, k)
                - atr(
                    lookback,
                    &bars.high,
                    &bars.low,
                    &bars.close,
                    k.saturating_sub(lookback),
                )
        } else {
            atr(lookback, &bars.high, &bars.low, &bars.close, k)
                - atr(full_lookback, &bars.high, &bars.low, &bars.close, k)
        };
        volatility.push(val);
    }
    volatility
}

pub fn calculate_expansion(
    bars: &BarData,
    lookback: usize,
    full_lookback: usize,
    version: i32,
) -> Vec<f64> {
    let nprices = bars.len();
    let nind = nprices - full_lookback + 1;
    let mut expansion = Vec::with_capacity(nind);

    for i in 0..nind {
        let k = full_lookback - 1 + i;
        let val = if version == 0 {
            range_expansion(lookback, k, &bars.close)
        } else if version == 1 {
            range_expansion(lookback, k, &bars.close)
                - range_expansion(lookback, k.saturating_sub(lookback), &bars.close)
        } else {
            range_expansion(lookback, k, &bars.close)
                - range_expansion(full_lookback, k, &bars.close)
        };
        expansion.push(val);
    }
    expansion
}

pub fn calculate_jump(
    bars: &BarData,
    lookback: usize,
    full_lookback: usize,
    version: i32,
) -> Vec<f64> {
    let nprices = bars.len();
    let nind = nprices - full_lookback + 1;
    let mut raw_jump = Vec::with_capacity(nind);

    for i in 0..nind {
        let k = full_lookback - 1 + i;
        let val = if version == 0 {
            jump(lookback, k, &bars.close)
        } else if version == 1 {
            jump(lookback, k, &bars.close) - jump(lookback, k.saturating_sub(lookback), &bars.close)
        } else {
            jump(lookback, k, &bars.close) - jump(full_lookback, k, &bars.close)
        };
        raw_jump.push(val);
    }
    raw_jump
}
