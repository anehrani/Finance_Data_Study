use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::BacktestResult;

/// Generate a text report
pub fn generate_text_report<P: AsRef<Path>>(result: &BacktestResult, path: P) -> Result<()> {
    let mut file = File::create(path)?;
    
    writeln!(file, "Backtest Report")?;
    writeln!(file, "===============")?;
    writeln!(file)?;
    
    writeln!(file, "Performance Metrics:")?;
    writeln!(file, "--------------------")?;
    
    // Sort keys for consistent output
    let mut keys: Vec<&String> = result.metrics.keys().collect();
    keys.sort();
    
    for key in keys {
        writeln!(file, "{}: {:.4}", key, result.metrics[key])?;
    }
    
    writeln!(file)?;
    writeln!(file, "Trades: {}", result.trades)?;
    
    Ok(())
}

/// Generate a JSON report
pub fn generate_json_report<P: AsRef<Path>>(result: &BacktestResult, path: P) -> Result<()> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, result)?;
    Ok(())
}
