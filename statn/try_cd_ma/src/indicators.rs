use anyhow::Result;
use indicators::trend::ma::compute_indicators as compute_ma_indicator;

/// Specification for a single indicator (MA crossover)
#[derive(Debug, Clone)]
pub struct IndicatorSpec {
    pub short_lookback: usize,
    pub long_lookback: usize,
}

/// Computed indicators and targets for a dataset
#[derive(Debug)]
pub struct IndicatorData {
    /// Indicator matrix: n_cases x n_vars
    pub data: Vec<f64>,
    /// Target returns: n_cases
    pub targets: Vec<f64>,
    /// Number of cases
    pub n_cases: usize,
    /// Number of variables (indicators)
    pub n_vars: usize,
}

/// Generate all indicator specifications based on configuration
pub fn generate_specs(lookback_inc: usize, n_long: usize, n_short: usize) -> Vec<IndicatorSpec> {
    (0..n_long)
        .flat_map(|ilong| {
            let long_lookback = (ilong + 1) * lookback_inc;
            (0..n_short).map(move |ishort| {
                let short_lookback = long_lookback * (ishort + 1) / (n_short + 1);
                let short_lookback = short_lookback.max(1);
                IndicatorSpec {
                    short_lookback,
                    long_lookback,
                }
            })
        })
        .collect()
}

/// Compute all indicators for a dataset
pub fn compute_all_indicators(
    prices: &[f64],
    start_idx: usize,
    n_cases: usize,
    specs: &[IndicatorSpec],
) -> Result<Vec<f64>> {
    let n_vars = specs.len();
    let mut data = vec![0.0; n_cases * n_vars];
    
    for (k, spec) in specs.iter().enumerate() {
        let indicators = compute_ma_indicator(
            n_cases,
            prices,
            start_idx,
            spec.short_lookback,
            spec.long_lookback,
        );
        
        for i in 0..n_cases {
            data[i * n_vars + k] = indicators[i];
        }
    }
    
    Ok(data)
}

/// Compute targets (returns) for a dataset
pub fn compute_targets(prices: &[f64], start_idx: usize, n_cases: usize) -> Vec<f64> {
    (0..n_cases)
        .map(|i| {
            let idx = start_idx + i;
            prices[idx + 1] - prices[idx]
        })
        .collect()
}

/// Compute both indicators and targets
pub fn compute_indicator_data(
    prices: &[f64],
    start_idx: usize,
    n_cases: usize,
    specs: &[IndicatorSpec],
) -> Result<IndicatorData> {
    let data = compute_all_indicators(prices, start_idx, n_cases, specs)?;
    let targets = compute_targets(prices, start_idx, n_cases);
    let n_vars = specs.len();
    
    Ok(IndicatorData {
        data,
        targets,
        n_cases,
        n_vars,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_specs() {
        let specs = generate_specs(10, 3, 2);
        assert_eq!(specs.len(), 6); // 3 * 2
        
        // Check first spec
        assert_eq!(specs[0].long_lookback, 10);
        assert!(specs[0].short_lookback > 0);
        
        // Check that long lookbacks increase
        assert!(specs[2].long_lookback > specs[0].long_lookback);
    }
    
    #[test]
    fn test_compute_targets() {
        let prices = vec![1.0, 1.1, 1.05, 1.15, 1.2];
        let targets = compute_targets(&prices, 0, 3);
        
        assert_eq!(targets.len(), 3);
        assert!((targets[0] - 0.1).abs() < 1e-10);
    }
}
