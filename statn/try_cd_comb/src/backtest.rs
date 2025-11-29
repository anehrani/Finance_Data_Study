use anyhow::Result;
use backtesting::core::backtest_signals;
use backtesting::models::{SignalResult, TradeStats};
use statn::models::cd_ma::CoordinateDescent;

/// Generate trading signals from model predictions
pub fn generate_signals(
    model: &CoordinateDescent,
    indicator_data: &[f64],
    n_cases: usize,
    n_vars: usize,
) -> Vec<i32> {
    let mut signals = Vec::with_capacity(n_cases);
    
    for i in 0..n_cases {
        let xptr = &indicator_data[i * n_vars..(i + 1) * n_vars];
        
        // Compute prediction
        let pred: f64 = xptr
            .iter()
            .enumerate()
            .map(|(ivar, &x)| {
                model.beta[ivar] * (x - model.xmeans[ivar]) / model.xscales[ivar]
            })
            .sum();
        
        let pred = pred * model.yscale + model.ymean;
        
        // Generate signal: 1 = BUY (long), -1 = SELL (short), 0 = HOLD
        let signal = if pred > 0.0 {
            1
        } else if pred < 0.0 {
            -1
        } else {
            0
        };
        
        signals.push(signal);
    }
    
    signals
}

/// Run backtest on test data
pub fn run_backtest(
    model: &CoordinateDescent,
    test_prices: &[f64],
    test_data: &[f64],
    n_cases: usize,
    n_vars: usize,
    initial_budget: f64,
    transaction_cost_pct: f64,
) -> Result<TradeStats> {
    // Generate signals
    let signals = generate_signals(model, test_data, n_cases, n_vars);
    
    // Create SignalResult
    let signal_result = SignalResult {
        prices: test_prices.to_vec(),
        signals,
        long_lookback: 0,  // Not used in backtesting
        short_pct: 0.0,    // Not used
        short_thresh: 0.0, // Not used
        long_thresh: 0.0,  // Not used
    };
    
    // Run backtest
    let stats = backtest_signals(&signal_result, initial_budget, transaction_cost_pct);
    
    Ok(stats)
}
