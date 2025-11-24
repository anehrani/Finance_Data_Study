use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::config::Config;
use crate::indicators::IndicatorSpec;
use crate::training::TrainingResult;
use statn::models::cd_ma::CoordinateDescent;

/// Evaluation results
#[derive(Debug)]
pub struct EvaluationResult {
    /// Out-of-sample total return (log)
    pub oos_return: f64,
    /// Out-of-sample return percentage
    pub oos_return_pct: f64,
    /// In-sample explained variance
    pub in_sample_explained: f64,
}

/// Evaluate model on test data
pub fn evaluate_model(
    model: &CoordinateDescent,
    test_data: &[f64],
    test_targets: &[f64],
    n_vars: usize,
) -> Result<EvaluationResult> {
    println!("Evaluating on test set...");
    
    let n_test = test_targets.len();
    
    let oos_return: f64 = (0..n_test)
        .map(|i| {
            let xptr = &test_data[i * n_vars..(i + 1) * n_vars];
            
            // Compute prediction
            let pred: f64 = xptr
                .iter()
                .enumerate()
                .map(|(ivar, &x)| {
                    model.beta[ivar] * (x - model.xmeans[ivar]) / model.xscales[ivar]
                })
                .sum();
            
            let pred = pred * model.yscale + model.ymean;
            
            // Trading logic: long if pred > 0, short if pred < 0
            if pred > 0.0 {
                test_targets[i]
            } else if pred < 0.0 {
                -test_targets[i]
            } else {
                0.0
            }
        })
        .sum();
    
    let oos_return_pct = 100.0 * (oos_return.exp() - 1.0);
    
    println!("OOS total return: {:.5} ({:.3}%)", oos_return, oos_return_pct);
    
    Ok(EvaluationResult {
        oos_return,
        oos_return_pct,
        in_sample_explained: model.explained,
    })
}

/// Write results to file
pub fn write_results<P: AsRef<Path>>(
    path: P,
    config: &Config,
    training: &TrainingResult,
    evaluation: &EvaluationResult,
    specs: &[IndicatorSpec],
) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path.as_ref())?;
    
    writeln!(file, "CD_MA - Moving Average Crossover Indicator Selection")?;
    writeln!(file, "{}", "=".repeat(60))?;
    writeln!(file)?;
    
    // Configuration
    writeln!(file, "Configuration:")?;
    writeln!(file, "  Lookback increment: {}", config.lookback_inc)?;
    writeln!(file, "  Number of long-term lookbacks: {}", config.n_long)?;
    writeln!(file, "  Number of short-term lookbacks: {}", config.n_short)?;
    writeln!(file, "  Alpha: {:.4}", config.alpha)?;
    writeln!(file, "  MA indicators: {}", config.n_ma_vars())?;
    if config.enable_rsi {
        writeln!(file, "  RSI indicators: {}", config.n_rsi_vars())?;
        writeln!(file, "  RSI periods: {:?}", config.rsi_periods)?;
    }
    writeln!(file, "  Total indicators: {}", config.n_vars())?;
    writeln!(file, "  Test cases: {}", config.n_test)?;
    writeln!(file)?;
    
    // Cross-validation results
    if config.alpha > 0.0 {
        writeln!(file, "Cross-Validation Results:")?;
        writeln!(file, "  Optimal lambda: {:.6}", training.lambda)?;
        writeln!(file)?;
        writeln!(file, "  {:>10} {:>15}", "Lambda", "OOS Explained")?;
        writeln!(file, "  {}", "-".repeat(27))?;
        for i in 0..training.lambdas.len() {
            writeln!(
                file,
                "  {:>10.4} {:>15.4}",
                training.lambdas[i], training.lambda_oos[i]
            )?;
        }
        writeln!(file)?;
    }
    
    // Beta coefficients for MA indicators
    writeln!(
        file,
        "MA Beta Coefficients (In-sample explained variance: {:.3}%):",
        100.0 * evaluation.in_sample_explained
    )?;
    writeln!(
        file,
        "Row: long-term lookback | Columns: short-term lookback (small to large)"
    )?;
    writeln!(file)?;
    
    let mut k = 0;
    for ilong in 0..config.n_long {
        let long_lookback = (ilong + 1) * config.lookback_inc;
        write!(file, "{:5} ", long_lookback)?;
        
        for _ishort in 0..config.n_short {
            if training.model.beta[k] != 0.0 {
                write!(file, "{:9.4}", training.model.beta[k])?;
            } else {
                write!(file, "    ----")?;
            }
            k += 1;
        }
        writeln!(file)?;
    }
    writeln!(file)?;
    
    // RSI beta coefficients if enabled
    if config.enable_rsi {
        writeln!(file, "RSI Beta Coefficients:")?;
        writeln!(file, "  {:>10} {:>15}", "Period", "Beta")?;
        writeln!(file, "  {}", "-".repeat(27))?;
        
        for (i, &period) in config.rsi_periods.iter().enumerate() {
            let beta_idx = config.n_ma_vars() + i;
            if training.model.beta[beta_idx] != 0.0 {
                writeln!(
                    file,
                    "  {:>10} {:>15.4}",
                    period, training.model.beta[beta_idx]
                )?;
            } else {
                writeln!(file, "  {:>10}         ----", period)?;
            }
        }
        writeln!(file)?;
    }
    
    // Out-of-sample results
    writeln!(file, "Out-of-Sample Results:")?;
    writeln!(
        file,
        "  Total return: {:.5} ({:.3}%)",
        evaluation.oos_return, evaluation.oos_return_pct
    )?;
    
    println!("\nResults written to {}", path.as_ref().display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use statn::models::cd_ma::CoordinateDescent;
    
    #[test]
    fn test_evaluate_model() {
        let n_vars = 3;
        let n_cases = 10;
        let mut model = CoordinateDescent::new(n_vars, n_cases, false, true, 0);
        
        // Set up dummy model parameters
        model.beta = vec![0.1, 0.2, -0.1];
        model.xmeans = vec![0.0; n_vars];
        model.xscales = vec![1.0; n_vars];
        model.ymean = 0.0;
        model.yscale = 1.0;
        model.explained = 0.5;
        
        let test_data = vec![0.0; n_vars * n_cases];
        let test_targets = vec![0.01; n_cases];
        
        let result = evaluate_model(&model, &test_data, &test_targets, n_vars);
        assert!(result.is_ok());
    }
}
