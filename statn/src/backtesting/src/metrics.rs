use rustc_hash::FxHashMap;

/// Calculate performance metrics
pub fn calculate_metrics(daily_returns: &[f64], risk_free_rate: f64) -> FxHashMap<String, f64> {
    let mut metrics = FxHashMap::default();
    let n = daily_returns.len();
    
    if n == 0 {
        return metrics;
    }

    // Total Return (cumulative)
    let total_return = daily_returns.iter().fold(1.0, |acc, r| acc * (1.0 + r)) - 1.0;
    metrics.insert("Total Return".to_string(), total_return);

    // Mean Daily Return
    let mean_return = daily_returns.iter().sum::<f64>() / n as f64;
    metrics.insert("Mean Daily Return".to_string(), mean_return);

    // Volatility (Standard Deviation of Daily Returns)
    let variance = daily_returns.iter().map(|r| (r - mean_return).powi(2)).sum::<f64>() / (n - 1) as f64;
    let volatility = variance.sqrt();
    metrics.insert("Daily Volatility".to_string(), volatility);
    metrics.insert("Annualized Volatility".to_string(), volatility * 252.0_f64.sqrt());

    // Sharpe Ratio (Annualized)
    // Assuming risk_free_rate is annual, convert to daily approx
    let daily_rf = (1.0 + risk_free_rate).powf(1.0 / 252.0) - 1.0;
    let excess_returns: Vec<f64> = daily_returns.iter().map(|r| r - daily_rf).collect();
    let mean_excess = excess_returns.iter().sum::<f64>() / n as f64;
    let std_excess = (excess_returns.iter().map(|r| (r - mean_excess).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt();
    
    if std_excess > 1e-9 {
        let sharpe = (mean_excess / std_excess) * 252.0_f64.sqrt();
        metrics.insert("Sharpe Ratio".to_string(), sharpe);
    } else {
        metrics.insert("Sharpe Ratio".to_string(), 0.0);
    }

    // Max Drawdown
    let mut max_drawdown = 0.0;
    let mut peak = 1.0;
    let mut current_value = 1.0;
    
    for r in daily_returns {
        current_value *= 1.0 + r;
        if current_value > peak {
            peak = current_value;
        }
        let drawdown = (peak - current_value) / peak;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }
    metrics.insert("Max Drawdown".to_string(), max_drawdown);

    metrics
}
