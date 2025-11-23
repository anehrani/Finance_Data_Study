use crate::random::{normal, unifrand};

/// Generate a set of trades using bootstrap sampling
pub fn get_trades(
    n_changes: usize,
    n_trades: usize,
    win_prob: f64,
    make_changes: bool,
    changes: &mut Vec<f64>,
    trades: &mut Vec<f64>,
) {
    if make_changes {
        changes.clear();
        for _ in 0..n_changes {
            let mut val = normal();
            if unifrand() < win_prob {
                val = val.abs();
            } else {
                val = -val.abs();
            }
            changes.push(val);
        }
    }

    // Bootstrap sample from changes
    trades.clear();
    for _ in 0..n_trades {
        let k = (unifrand() * n_changes as f64) as usize;
        let k = k.min(n_changes - 1);
        trades.push(changes[k]);
    }
}

/// Compute mean return
pub fn mean_return(trades: &[f64]) -> f64 {
    trades.iter().sum::<f64>() / trades.len() as f64
}

/// Compute drawdown
pub fn drawdown(trades: &[f64]) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }

    let mut cumulative = trades[0];
    let mut max_price = cumulative;
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

    dd
}

/// Compute drawdown quantiles using bootstrap
pub fn drawdown_quantiles(
    n_changes: usize,
    n_trades: usize,
    b_changes: &[f64],
    nboot: usize,
    bootsample: &mut Vec<f64>,
    work: &mut Vec<f64>,
) -> (f64, f64, f64, f64) {
    work.clear();

    for _ in 0..nboot {
        bootsample.clear();
        for _ in 0..n_trades {
            let k = (unifrand() * n_changes as f64) as usize;
            let k = k.min(n_changes - 1);
            bootsample.push(b_changes[k]);
        }
        work.push(drawdown(bootsample));
    }

    work.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let q001 = find_quantile(work, 0.999);
    let q01 = find_quantile(work, 0.99);
    let q05 = find_quantile(work, 0.95);
    let q10 = find_quantile(work, 0.90);

    (q001, q01, q05, q10)
}

/// Find a quantile from sorted data
pub fn find_quantile(data: &[f64], frac: f64) -> f64 {
    let k = ((frac * (data.len() + 1) as f64) as usize).saturating_sub(1);
    let k = k.min(data.len() - 1);
    data[k]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::set_seed;

    #[test]
    fn test_mean_return() {
        let trades = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(mean_return(&trades), 3.0);
    }

    #[test]
    fn test_drawdown_no_loss() {
        let trades = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(drawdown(&trades), 0.0);
    }

    #[test]
    fn test_drawdown_with_loss() {
        let trades = vec![1.0, 2.0, -1.0, -1.0];
        // Cumulative: 1, 3, 2, 1
        // Max: 1, 3, 3, 3
        // DD: 0, 0, 1, 2
        assert_eq!(drawdown(&trades), 2.0);
    }

    #[test]
    fn test_find_quantile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        assert_eq!(find_quantile(&data, 0.5), 5.0);
        assert_eq!(find_quantile(&data, 0.9), 9.0);
    }

    #[test]
    fn test_get_trades() {
        set_seed(12345);
        let mut changes = Vec::new();
        let mut trades = Vec::new();
        
        get_trades(100, 50, 0.5, true, &mut changes, &mut trades);
        
        assert_eq!(changes.len(), 100);
        assert_eq!(trades.len(), 50);
    }
}
