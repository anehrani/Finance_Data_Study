use anyhow::Result;
use indicators::trend::ma::compute_indicators as compute_ma_indicator;
use indicators::oscillators::rsi::rsi;
use indicators::oscillators::macd::{macd_histogram, MacdConfig, ema};
use statn::core::io::compute_targets;

use serde::{Deserialize, Serialize};

/// Specification for an indicator
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CrossoverType {
    Ma,
    Rsi,
    Ema,
    Macd,
    Roc,
}

#[derive(Debug, Clone)]
pub enum IndicatorSpec {
    Crossover {
        type_: CrossoverType,
        short_lookback: usize,
        long_lookback: usize,
    },
    Rsi {
        period: usize,
    },
    Macd {
        fast: usize,
        slow: usize,
        signal: usize,
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
    macd_configs: &[(usize, usize, usize)],
    crossover_types: &[CrossoverType],
) -> Vec<IndicatorSpec> {
    let mut specs = Vec::new();

    // Crossovers
    for &ctype in crossover_types {
        for ilong in 0..n_long {
            let long_lookback = (ilong + 1) * lookback_inc;
            for ishort in 0..n_short {
                let short_lookback = long_lookback * (ishort + 1) / (n_short + 1);
                let short_lookback = short_lookback.max(1);
                specs.push(IndicatorSpec::Crossover {
                    type_: ctype,
                    short_lookback,
                    long_lookback,
                });
            }
        }
    }

    // RSI
    for &period in rsi_periods {
        specs.push(IndicatorSpec::Rsi { period });
    }

    // MACD
    for &(fast, slow, signal) in macd_configs {
        specs.push(IndicatorSpec::Macd { fast, slow, signal });
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

            IndicatorSpec::Crossover { type_, short_lookback, long_lookback } => {
                match type_ {
                    CrossoverType::Ma => compute_ma_indicator(
                        n_cases,
                        prices,
                        start_idx,
                        *short_lookback,
                        *long_lookback,
                    ),
                    CrossoverType::Rsi => {
                        let short_rsi = rsi(prices, *short_lookback);
                        let long_rsi = rsi(prices, *long_lookback);
                        
                        let mut inds = vec![0.0; n_cases];
                        for i in 0..n_cases {
                            let idx = start_idx + i;
                            if idx < short_rsi.len() && idx < long_rsi.len() {
                                inds[i] = short_rsi[idx] - long_rsi[idx];
                            } else {
                                inds[i] = f64::NAN;
                            }
                        }
                        inds
                    },
                    CrossoverType::Ema => {
                        let short_ema = ema(prices, *short_lookback);
                        let long_ema = ema(prices, *long_lookback);
                        
                        let mut inds = vec![0.0; n_cases];
                        for i in 0..n_cases {
                            let idx = start_idx + i;
                            if idx < short_ema.len() && idx < long_ema.len() {
                                inds[i] = short_ema[idx] - long_ema[idx];
                            } else {
                                inds[i] = f64::NAN;
                            }
                        }
                        inds
                    },
                    CrossoverType::Macd => {
                        // Use short as fast, long as slow, fixed signal=9
                        // Note: MACD requires fast < slow usually, but we'll let the grid handle it.
                        // If fast >= slow, it might be weird but valid math.
                        let config = MacdConfig {
                            fast_period: *short_lookback,
                            slow_period: *long_lookback,
                            signal_period: 9,
                        };
                        let hist = macd_histogram(prices, config);
                        
                        let mut inds = vec![0.0; n_cases];
                        for i in 0..n_cases {
                            let idx = start_idx + i;
                            if idx < hist.len() {
                                inds[i] = hist[idx];
                            } else {
                                inds[i] = f64::NAN;
                            }
                        }
                        inds
                    },
                    CrossoverType::Roc => {
                        let short_roc = roc(prices, *short_lookback);
                        let long_roc = roc(prices, *long_lookback);
                        
                        let mut inds = vec![0.0; n_cases];
                        for i in 0..n_cases {
                            let idx = start_idx + i;
                            if idx < short_roc.len() && idx < long_roc.len() {
                                inds[i] = short_roc[idx] - long_roc[idx];
                            } else {
                                inds[i] = f64::NAN;
                            }
                        }
                        inds
                    }
                }
            },
            IndicatorSpec::Rsi { period } => {
                let full_rsi = rsi(prices, *period);
                // Extract the relevant slice
                if start_idx + n_cases > full_rsi.len() {
                    return Err(anyhow::anyhow!("RSI computation out of bounds"));
                }
                full_rsi[start_idx..start_idx + n_cases].to_vec()
            },
            IndicatorSpec::Macd { fast, slow, signal } => {
                // Compute MACD histogram for entire price series
                let config = MacdConfig {
                    fast_period: *fast,
                    slow_period: *slow,
                    signal_period: *signal,
                };
                let full_macd = macd_histogram(prices, config);
                // Extract the relevant slice
                if start_idx + n_cases > full_macd.len() {
                    return Err(anyhow::anyhow!("MACD computation out of bounds"));
                }
                full_macd[start_idx..start_idx + n_cases].to_vec()
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

fn roc(data: &[f64], period: usize) -> Vec<f64> {
    if period == 0 || period >= data.len() {
        return vec![f64::NAN; data.len()];
    }
    
    let mut roc_values = vec![f64::NAN; data.len()];
    for i in period..data.len() {
        if data[i - period] != 0.0 {
            roc_values[i] = (data[i] - data[i - period]) / data[i - period];
        }
    }
    roc_values
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_specs() {
        // Test with RSI and multiple MACD configs
        let macd_configs = vec![(12, 26, 9), (5, 35, 5)];
        let crossover_types = vec![CrossoverType::Ma];
        let specs = generate_specs(10, 3, 2, &[14], &macd_configs, &crossover_types);
        assert_eq!(specs.len(), 9); // 3 * 2 + 1 RSI + 2 MACD
        
        // Check first spec (MA)
        if let IndicatorSpec::Crossover { type_, long_lookback, .. } = &specs[0] {
            assert_eq!(*type_, CrossoverType::Ma);
            assert_eq!(*long_lookback, 10);
        } else {
            panic!("Expected MA crossover");
        }
        
        // Check RSI spec
        if let IndicatorSpec::Rsi { period } = specs[6] {
            assert_eq!(period, 14);
        } else {
            panic!("Expected RSI");
        }
        
        // Check first MACD spec
        if let IndicatorSpec::Macd { fast, slow, signal } = specs[7] {
            assert_eq!(fast, 12);
            assert_eq!(slow, 26);
            assert_eq!(signal, 9);
        } else {
            panic!("Expected MACD");
        }
        
        // Check second MACD spec
        if let IndicatorSpec::Macd { fast, slow, signal } = specs[8] {
            assert_eq!(fast, 5);
            assert_eq!(slow, 35);
            assert_eq!(signal, 5);
        } else {
            panic!("Expected MACD");
        }
        
        // Test without MACD
        let specs_no_macd = generate_specs(10, 3, 2, &[14], &[], &crossover_types);
        assert_eq!(specs_no_macd.len(), 7); // 3 * 2 + 1 RSI, no MACD

        // Test with RSI Crossover
        let crossover_types_rsi = vec![CrossoverType::Rsi];
        let specs_rsi_cross = generate_specs(10, 3, 2, &[], &[], &crossover_types_rsi);
        assert_eq!(specs_rsi_cross.len(), 6); // 3 * 2
        if let IndicatorSpec::Crossover { type_, .. } = &specs_rsi_cross[0] {
            assert_eq!(*type_, CrossoverType::Rsi);
        } else {
            panic!("Expected RSI crossover");
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
