use crate::random::Rng;
use crate::sort::qsortd;

/// Compute drawdown from trades (assumes log equity changes)
/// Returns percent drawdown
pub fn drawdown(trades: &[f64]) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }

    let mut cumulative = trades[0];
    let mut max_price = trades[0];
    let mut dd = 0.0;

    for &trade in &trades[1..] {
        cumulative += trade;
        if cumulative > max_price {
            max_price = cumulative;
        } else {
            let loss = max_price - cumulative;
            if loss > dd {
                dd = loss;
            }
        }
    }

    100.0 * (1.0 - (-dd).exp()) // Convert log change to percent
}

/// Compute four drawdown quantiles using bootstrap
#[allow(clippy::too_many_arguments)]
pub fn drawdown_quantiles(
    n_changes: usize,
    n_trades: usize,
    b_changes: &[f64],
    nboot: usize,
    quantsample: &mut [f64],
    work: &mut [f64],
    rng: &mut Rng,
) -> (f64, f64, f64, f64) {
    for iboot in 0..nboot {
        for i in 0..n_trades {
            let k = (rng.unifrand() * n_changes as f64) as usize;
            let k = if k >= n_changes { n_changes - 1 } else { k };
            quantsample[i] = b_changes[k];
        }
        work[iboot] = drawdown(quantsample);
    }

    qsortd(work);

    let q001 = find_quantile(nboot, work, 0.999);
    let q01 = find_quantile(nboot, work, 0.99);
    let q05 = find_quantile(nboot, work, 0.95);
    let q10 = find_quantile(nboot, work, 0.90);

    (q001, q01, q05, q10)
}

/// Find a quantile from sorted data
pub fn find_quantile(n: usize, data: &[f64], frac: f64) -> f64 {
    let k = ((frac * (n + 1) as f64) as usize).saturating_sub(1);
    let k = k.min(n.saturating_sub(1));
    data[k]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawdown() {
        let trades = vec![0.01, 0.02, -0.05, 0.01, 0.02];
        let dd = drawdown(&trades);
        assert!(dd > 0.0);
    }

    #[test]
    fn test_find_quantile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let q50 = find_quantile(5, &data, 0.5);
        assert!((q50 - 3.0).abs() < 1e-10);
    }
}
