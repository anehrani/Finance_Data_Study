use anyhow::Result;
use indicators::trend::ma::compute_indicators as compute_ma_indicator;
use indicators::oscillator::rsi::compute_rsi_ema;
use statn::core::io::compute_targets;

/// Specification for a single indicator
#[derive(Debug, Clone)]
pub enum IndicatorSpec {
    /// Moving average crossover indicator
    MovingAverage {
        short_lookback: usize,
        long_lookback: usize,
    },
    /// RSI oscillation indicator
    RSI {
        period: usize,
    },
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
pub fn generate_specs(
    lookback_inc: usize,
    n_long: usize,
    n_short: usize,
    enable_rsi: bool,
    rsi_periods: &[usize],
) -> Vec<IndicatorSpec> {
    let mut specs = Vec::new();
    
    // Generate MA crossover indicators
    for ilong in 0..n_long {
        let long_lookback = (ilong + 1) * lookback_inc;
        for ishort in 0..n_short {
            let short_lookback = long_lookback * (ishort + 1) / (n_short + 1);
            let short_lookback = short_lookback.max(1);
            specs.push(IndicatorSpec::MovingAverage {
                short_lookback,
                long_lookback,
            });
        }
    }
    
    // Generate RSI indicators if enabled
    if enable_rsi {
        for &period in rsi_periods {
            specs.push(IndicatorSpec::RSI { period });
        }
    }
    
    specs
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
        let indicators = match spec {
            IndicatorSpec::MovingAverage { short_lookback, long_lookback } => {
                compute_ma_indicator(
                    n_cases,
                    prices,
                    start_idx,
                    *short_lookback,
                    *long_lookback,
                )
            }
            IndicatorSpec::RSI { period } => {
                compute_rsi_ema(
                    n_cases,
                    prices,
                    start_idx,
                    *period,
                )
            }
        };
        
        for i in 0..n_cases {
            data[i * n_vars + k] = indicators[i];
        }
    }
    
    Ok(data)
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
        let specs = generate_specs(10, 3, 2, false, &[]);
        assert_eq!(specs.len(), 6); // 3 * 2
        
        // Check first spec is MA
        match &specs[0] {
            IndicatorSpec::MovingAverage { short_lookback, long_lookback } => {
                assert_eq!(*long_lookback, 10);
                assert!(*short_lookback > 0);
            }
            _ => panic!("Expected MovingAverage spec"),
        }
        
        // Check that long lookbacks increase
        match (&specs[0], &specs[2]) {
            (
                IndicatorSpec::MovingAverage { long_lookback: l1, .. },
                IndicatorSpec::MovingAverage { long_lookback: l2, .. },
            ) => {
                assert!(l2 > l1);
            }
            _ => panic!("Expected MovingAverage specs"),
        }
    }
    
    #[test]
    fn test_generate_specs_with_rsi() {
        let specs = generate_specs(10, 3, 2, true, &[7, 14, 21]);
        assert_eq!(specs.len(), 9); // 3 * 2 + 3
        
        // First 6 should be MA
        for i in 0..6 {
            match &specs[i] {
                IndicatorSpec::MovingAverage { .. } => {},
                _ => panic!("Expected MovingAverage spec at index {}", i),
            }
        }
        
        // Last 3 should be RSI
        for i in 6..9 {
            match &specs[i] {
                IndicatorSpec::RSI { .. } => {},
                _ => panic!("Expected RSI spec at index {}", i),
            }
        }
    }
    
    #[test]
    fn test_compute_targets() {
        let prices = vec![1.0, 1.1, 1.05, 1.15, 1.2];
        let targets = compute_targets(&prices, 0, 3);
        
        assert_eq!(targets.len(), 3);
        assert!((targets[0] - 0.1).abs() < 1e-10);
    }
}
