use anyhow::Result;
use indicators::trend::ma::compute_indicators as compute_ma_indicator;
use indicators::oscillators::rsi::rsi;
use statn::core::io::compute_targets;

/// Specification for an indicator
#[derive(Debug, Clone)]
pub enum IndicatorSpec {
    MaCrossover {
        short_lookback: usize,
        long_lookback: usize,
    },
    Rsi {
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
    rsi_periods: &[usize],
) -> Vec<IndicatorSpec> {
    let mut specs = Vec::new();

    // MA Crossovers
    for ilong in 0..n_long {
        let long_lookback = (ilong + 1) * lookback_inc;
        for ishort in 0..n_short {
            let short_lookback = long_lookback * (ishort + 1) / (n_short + 1);
            let short_lookback = short_lookback.max(1);
            specs.push(IndicatorSpec::MaCrossover {
                short_lookback,
                long_lookback,
            });
        }
    }

    // RSI
    for &period in rsi_periods {
        specs.push(IndicatorSpec::Rsi { period });
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
            IndicatorSpec::MaCrossover { short_lookback, long_lookback } => {
                compute_ma_indicator(
                    n_cases,
                    prices,
                    start_idx,
                    *short_lookback,
                    *long_lookback,
                )
            },
            IndicatorSpec::Rsi { period } => {
                let full_rsi = rsi(prices, *period);
                // Extract the relevant slice. Note: rsi returns full vector aligned with prices.
                // We need [start_idx .. start_idx + n_cases]
                if start_idx + n_cases > full_rsi.len() {
                    return Err(anyhow::anyhow!("RSI computation out of bounds"));
                }
                full_rsi[start_idx..start_idx + n_cases].to_vec()
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
        let specs = generate_specs(10, 3, 2, &[14]);
        assert_eq!(specs.len(), 7); // 3 * 2 + 1
        
        // Check first spec (MA)
        if let IndicatorSpec::MaCrossover { long_lookback, .. } = specs[0] {
            assert_eq!(long_lookback, 10);
        } else {
            panic!("Expected MA crossover");
        }
        
        // Check last spec (RSI)
        if let IndicatorSpec::Rsi { period } = specs[6] {
            assert_eq!(period, 14);
        } else {
            panic!("Expected RSI");
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
